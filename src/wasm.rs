use crate::errors::CastError;
use serde_json::Value;
use std::path::PathBuf;
use wasmtime::*;

pub struct WasmRuntime {
    engine: Engine,
    module_path: PathBuf,
}

impl WasmRuntime {
    pub fn new(module_path: &str) -> Self {
        let config = Config::new();
        let engine = Engine::new(&config).expect("Failed to create WASM engine");

        Self {
            engine,
            module_path: PathBuf::from(module_path),
        }
    }

    pub fn execute_spell(&self, spell_name: &str, input: Value) -> Result<Value, CastError> {
        let wasm_file = self.module_path.join(format!("{}.wasm", spell_name));

        if !wasm_file.exists() {
            return Err(CastError::WasmNotFound(spell_name.to_string()));
        }

        let module = Module::from_file(&self.engine, &wasm_file)
            .map_err(|e| CastError::WasmExecutionFailed(format!("Failed to load module: {}", e)))?;

        let mut store = Store::new(&self.engine, ());

        let linker = Linker::new(&self.engine);

        let _instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| CastError::WasmExecutionFailed(format!("Failed to instantiate: {}", e)))?;

        // For now, return mock success response
        // In a real implementation, we would:
        // 1. Call exported function with input
        // 2. Parse memory to get output
        // 3. Convert to JSON

        Ok(serde_json::json!({
            "spell": spell_name,
            "input": input,
            "output": "WASM execution successful (mock)",
            "status": "ok"
        }))
    }
}
