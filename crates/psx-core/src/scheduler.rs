#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScheduleKey {
    pub tick: u64,
}

impl ScheduleKey {
    pub fn new(tick: u64) -> Self {
        ScheduleKey { tick }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scheduler {
    events: Vec<(ScheduleKey, EventId)>,
    current_tick: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            events: Vec::new(),
            current_tick: 0,
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn schedule(&mut self, key: ScheduleKey, id: EventId) {
        self.events.push((key, id));
        self.events.sort_by_key(|a| a.0.tick);
    }

    pub fn cancel(&mut self, id: EventId) {
        self.events.retain(|(_, e)| *e != id);
    }

    // Devolve o PRAZO que venceu junto do evento, nao so o evento: quem reagenda um evento
    // periodico precisa contar o proximo prazo a partir de QUANDO ELE VENCEU, nao de
    // `ticks` (o instante em que o chamador percebeu) -- senao um tick grande que cobre
    // varios periodos reagenda o unico evento restante la na frente e perde os demais.
    pub fn advance_to(&mut self, ticks: u64) -> Option<(u64, EventId)> {
        self.current_tick = ticks;
        if self.events.is_empty() {
            return None;
        }
        let next_tick = self.events[0].0.tick;
        if next_tick > self.current_tick {
            return None;
        }
        let (key, id) = self.events.remove(0);
        Some((key.tick, id))
    }

    pub fn pending_events(&self) -> &[(ScheduleKey, EventId)] {
        &self.events
    }
}
