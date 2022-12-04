#[derive(Default)]
pub struct Parameter<T> {
    value: T,
    def_value: T,
    name: &'static str,
    description: &'static str,
}

impl<T> Parameter<T> {
    pub fn set(&mut self, v: T) -> &mut Self {
        self.value = v;
        self
    }

    pub fn name(&mut self, n: &'static str) -> &mut Self {
        self.name = n;

        self
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn default(&mut self, v: T) -> &mut Self {
        self.def_value = v;

        self
    }

    pub fn description(&mut self, d: &'static str) -> &mut Self {
        self.description = d;

        self
    }
}
