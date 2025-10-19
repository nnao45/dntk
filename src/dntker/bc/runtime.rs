use std::collections::{BTreeMap, HashMap};

use dashu::Decimal;
use rand::{rngs::SmallRng, SeedableRng};

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub params: Vec<String>,
    pub body: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum StatementOutcome {
    None,
    Value(Decimal),
    Return(Decimal),
}

impl StatementOutcome {
    pub fn value(value: Decimal) -> Self {
        StatementOutcome::Value(value)
    }

    pub fn ret(value: Decimal) -> Self {
        StatementOutcome::Return(value)
    }
}

#[derive(Debug)]
pub struct Runtime {
    namespaces: Vec<BTreeMap<String, Decimal>>,
    functions: HashMap<String, FunctionDef>,
    scale: u32,
    obase: u32,
    rng: SmallRng,
}

impl Runtime {
    pub fn with_defaults(scale: u32) -> Self {
        let mut namespaces = vec![BTreeMap::new()];
        namespaces[0].insert("scale".to_string(), Decimal::from(scale));
        namespaces[0].insert("obase".to_string(), Decimal::from(10));
        Self {
            namespaces,
            functions: HashMap::new(),
            scale,
            obase: 10,
            rng: SmallRng::seed_from_u64(0x5eed_5eed_5eed_5eed),
        }
    }

    pub fn scale(&self) -> u32 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: u32) {
        self.scale = scale;
        if let Some(scope) = self.namespaces.last_mut() {
            scope.insert("scale".to_string(), Decimal::from(scale));
        }
    }

    pub fn obase(&self) -> u32 {
        self.obase
    }

    pub fn set_obase(&mut self, obase: u32) {
        self.obase = obase;
        if let Some(scope) = self.namespaces.last_mut() {
            scope.insert("obase".to_string(), Decimal::from(obase));
        }
    }

    pub fn rng_mut(&mut self) -> &mut SmallRng {
        &mut self.rng
    }

    pub fn push_scope(&mut self, scope: BTreeMap<String, Decimal>) {
        self.namespaces.push(scope);
    }

    pub fn pop_scope(&mut self) {
        if self.namespaces.len() > 1 {
            self.namespaces.pop();
        }
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut BTreeMap<String, Decimal>> {
        self.namespaces.last_mut()
    }

    pub fn find_scope_mut(&mut self, name: &str) -> Option<&mut BTreeMap<String, Decimal>> {
        self.namespaces
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
    }

    pub fn get_variable(&self, name: &str) -> Option<Decimal> {
        self.namespaces
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    pub fn define_function(&mut self, name: String, def: FunctionDef) {
        self.functions.insert(name, def);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }

    pub fn reseed_rng(&mut self, seed: u64) {
        self.rng = SmallRng::seed_from_u64(seed);
    }
}
