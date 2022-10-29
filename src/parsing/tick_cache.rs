struct TickCache {
    // Tick (left, right)
    ticks: Vec<(i32, (i32, i32))>,
}

impl TickCache {
    pub fn new() -> Self {
        TickCache { ticks: vec![] }
    }
    pub fn insert_tick(&mut self, tick: i32, left: i32, right: i32) {
        self.ticks
    }
}
