//! Capabilities Gating con Cache en Memoria
//! Valida que las operaciones estén permitidas sin re-leer la DB en cada petición.

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Capabilities disponibles en un proyecto TAS
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    pub universes: bool,
    pub bestiary: bool,
    pub locations: bool,
    pub timeline: bool,
    pub boards: bool,
    pub forge: bool,
    pub snapshots: bool,
    pub trash: bool,
}

impl Capabilities {
    /// Crea capabilities con todo habilitado (default seguro)
    pub fn all_enabled() -> Self {
        Self {
            universes: true,
            bestiary: true,
            locations: true,
            timeline: true,
            boards: true,
            forge: true,
            snapshots: true,
            trash: true,
        }
    }

    /// Verifica si una capability está habilitada
    pub fn is_enabled(&self, capability: &str) -> bool {
        // Acepta aliases canónicos (canon v2) sin tocar el JSON almacenado en db_meta.
        // Canon: novel, pm, worldbuilding, timeline, trash, snapshots
        // Interno actual: forge, boards, universes, bestiary, locations, timeline, trash, snapshots
        let cap = capability.trim().to_ascii_lowercase();

        match cap.as_str() {
            // Canon → interno
            "novel" | "the_forge" => self.forge,
            "pm" | "project_management" => self.boards,
            "worldbuilding" | "world_building" | "world-building" => {
                self.universes || self.bestiary || self.locations
            }

            // Internos (tal cual)
            "forge" => self.forge,
            "boards" => self.boards,
            "universes" | "universe" => self.universes,
            "bestiary" | "creatures" => self.bestiary,
            "locations" | "location" => self.locations,
            "timeline" => self.timeline,
            "trash" => self.trash,
            "snapshots" => self.snapshots,

            _ => false,
        }
    }
}

/// Contenedor thread-safe para capabilities
pub type CapabilitiesCache = Arc<RwLock<Capabilities>>;

/// Error cuando una capability está deshabilitada
#[derive(Debug)]
pub struct CapabilityDisabledError {
    pub capability: String,
}

impl std::fmt::Display for CapabilityDisabledError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Capability '{}' is disabled in this project",
            self.capability
        )
    }
}

impl std::error::Error for CapabilityDisabledError {}

/// CARGA INICIAL: Llama a esto al arrancar la app para llenar el cache
pub async fn fetch_capabilities_from_db(pool: &SqlitePool) -> Result<Capabilities, sqlx::Error> {
    // Nota: db_meta (según tus migraciones) NO tiene columna "id".
    // Por eso usamos LIMIT 1.
    let row = sqlx::query(
        "SELECT enabled_capabilities_json
         FROM db_meta
         LIMIT 1"
    )
        .fetch_one(pool)
        .await?;

    let json: String = row.try_get("enabled_capabilities_json")?;

    // ✅ FAIL-CLOSED:
    // Soporta 2 formatos:
    // 1) NUEVO (recomendado): ["worldbuilding","timeline","pm","novel","snapshots","trash"]
    // 2) LEGACY: { "universes": true, "bestiary": true, ... }
    //
    // Si está corrupto → todo deshabilitado.
    let enabled_keys: Vec<String> = match serde_json::from_str::<Vec<String>>(&json) {
        Ok(v) => v,
        Err(_) => {
            // Intento #2: legacy object map
            match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(serde_json::Value::Object(map)) => {
                    let mut keys = Vec::new();
                    for (k, v) in map {
                        if v.as_bool().unwrap_or(false) {
                            keys.push(k);
                        }
                    }
                    keys
                }
                Ok(serde_json::Value::Array(arr)) => {
                    // Por si viene como array pero no matcheó Vec<String> (valores raros)
                    let mut keys = Vec::new();
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            keys.push(s.to_string());
                        }
                    }
                    keys
                }
                Ok(_) | Err(_) => {
                    crate::logger::warn(&format!(
                        "⚠️ enabled_capabilities_json invalid/corrupt. Using fail-closed defaults (all disabled). Raw: {}",
                        json
                    ));
                    return Ok(Capabilities::default());
                }
            }
        }
    };

    let mut caps = Capabilities::default();

    for cap in enabled_keys {
        match cap.as_str() {
            // -------------------------
            // INTERNAL KEYS (current)
            // -------------------------
            "universes" => caps.universes = true,
            "bestiary" => caps.bestiary = true,
            "locations" => caps.locations = true,
            "timeline" => caps.timeline = true,
            "boards" => caps.boards = true,
            "forge" => caps.forge = true,
            "snapshots" => caps.snapshots = true,
            "trash" => caps.trash = true,

            // -------------------------
            // CANON v2 ALIASES (safe)
            // -------------------------
            "novel" => caps.forge = true,
            "pm" => caps.boards = true,
            "worldbuilding" => {
                caps.universes = true;
                caps.bestiary = true;
                caps.locations = true;
            }

            _ => {
                crate::logger::warn(&format!(
                    "⚠️ Unknown capability key '{}' ignored",
                    cap
                ));
            }
        }
    }

    Ok(caps)
}

/// VERIFICACIÓN ULTRA-RÁPIDA: Usa el cache en memoria
pub async fn check_capability(
    cache: &CapabilitiesCache,
    capability: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Lectura compartida (múltiples hilos pueden leer a la vez)
    let caps = cache.read().await;

    if !caps.is_enabled(capability) {
        return Err(Box::new(CapabilityDisabledError {
            capability: capability.to_string(),
        }));
    }

    Ok(())
}

/// Helper: Crear cache inicial vacío (antes de cargar DB)
pub fn create_empty_cache() -> Arc<RwLock<Capabilities>> {
    // Fail-closed default: everything disabled until proven enabled by DB
    Arc::new(RwLock::new(Capabilities::default()))
}

/// Macro helper simplificada
#[macro_export]
macro_rules! require_capability {
    ($cache:expr, $cap:expr) => {
        $crate::guards::check_capability($cache, $cap).await?
    };
}