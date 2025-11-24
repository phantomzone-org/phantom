use std::time::{Duration, Instant};

pub struct Measurements {
    pub cycle_measurements: Vec<Measurement>,
}

#[allow(dead_code)]
pub(crate) fn average_time(times: &[Duration]) -> Duration {
    let total_time: Duration = times.iter().map(|c| c).sum::<Duration>();
    total_time / times.len() as u32
}

impl Measurements {
    pub fn new() -> Self {
        Self {
            cycle_measurements: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn average_cycle_time(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.total_cycle_time)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_prepare_pc(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_prepare_pc)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_read_and_prepare_rom(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_read_and_prepare_rom)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_read_rom(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_read_rom)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_prepare_rom(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_prepare_rom)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_read_and_prepare_registers(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_read_and_prepare_registers)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_read_registers(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_read_registers)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_prepare_registers(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_prepare_registers)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_read_ram(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_read_ram)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_derive_ram_addr(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_derive_ram_addr)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_prepare_ram_addr(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_prepare_ram_addr)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_prepare_ram_read_statefull(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_prepare_ram_read_statefull)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_update_registers(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_update_registers)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_evaluate_rd_ops(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_evaluate_rd_ops)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_blind_select_rd(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_blind_select_rd)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_refresh_rd(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_refresh_rd)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_write_rd(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_write_rd)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_update_ram(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_update_ram)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_ram_update_op_eval(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_ram_update_op_eval)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_blind_select_ram_value(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_blind_select_ram_value)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_refresh_ram_value(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_refresh_ram_value)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_write_ram(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_write_ram)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn average_time_update_pc(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.time_update_pc)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    #[allow(dead_code)]
    pub fn get_pc_val_fhe_uint_noise_list(&self) -> Vec<f64> {
        self.cycle_measurements
            .iter()
            .map(|measurement| measurement.pc_val_fhe_uint_noise)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_imm_val_fhe_uint_noise_list(&self) -> Vec<f64> {
        self.cycle_measurements
            .iter()
            .map(|measurement| measurement.imm_val_fhe_uint_noise)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_ram_addr_read_noise_list(&self) -> Vec<f64> {
        self.cycle_measurements
            .iter()
            .map(|measurement| measurement.ram_addr_read_noise)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_ram_val_read_noise_list(&self) -> Vec<f64> {
        self.cycle_measurements
            .iter()
            .map(|measurement| measurement.ram_val_read_noise)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_rd_val_fhe_uint_noise_list(&self) -> Vec<f64> {
        self.cycle_measurements
            .iter()
            .map(|measurement| measurement.rd_val_fhe_uint_noise)
            .collect()
    }
}

pub struct Measurement {
    // Layer zero
    pub total_cycle_time: Duration,

    // 1) Prepare PC
    pub time_prepare_pc: Duration,

    // 2) Read & prepare ROM (dep 1)
    pub time_read_and_prepare_rom: Duration,
    pub time_read_rom: Duration,
    pub time_prepare_rom: Duration,

    // 3) Read & prepare REGSITERS (dep 2)
    pub time_read_and_prepare_registers: Duration,
    pub time_read_registers: Duration,
    pub time_prepare_registers: Duration,

    // 4) Read RAM (dep 3)
    pub time_read_ram: Duration,
    pub time_derive_ram_addr: Duration,
    pub time_prepare_ram_addr: Duration,
    pub time_prepare_ram_read_statefull: Duration,

    // 4) Update REGISTERS (dep 4)
    pub time_update_registers: Duration,
    pub time_evaluate_rd_ops: Duration,
    pub time_blind_select_rd: Duration,
    pub time_refresh_rd: Duration,
    pub time_write_rd: Duration,

    // 5) Update RAM (dep 4)
    pub time_update_ram: Duration,
    pub time_ram_update_op_eval: Duration,
    pub time_blind_select_ram_value: Duration,
    pub time_refresh_ram_value: Duration,
    pub time_write_ram: Duration,

    // 6) Update PC (dep 4)
    pub time_update_pc: Duration,

    pub pc_val_fhe_uint_noise: f64,
    pub imm_val_fhe_uint_noise: f64,

    pub ram_addr_read_noise: f64,
    pub ram_val_read_noise: f64,

    pub rd_val_fhe_uint_noise: f64,
}

impl Measurement {
    pub fn new() -> Self {
        Self {
            total_cycle_time: Duration::from_secs(0),

            // 1) Prepare PC
            time_prepare_pc: Duration::from_secs(0),

            // 2) Read & prepare ROM (dep 1)
            time_read_and_prepare_rom: Duration::from_secs(0),
            time_read_rom: Duration::from_secs(0),
            time_prepare_rom: Duration::from_secs(0),

            // 3) Read & prepare REGSITERS (dep 2)
            time_read_and_prepare_registers: Duration::from_secs(0),
            time_read_registers: Duration::from_secs(0),
            time_prepare_registers: Duration::from_secs(0),

            // 4) Read RAM (dep 3)
            time_read_ram: Duration::from_secs(0),
            time_derive_ram_addr: Duration::from_secs(0),
            time_prepare_ram_addr: Duration::from_secs(0),
            time_prepare_ram_read_statefull: Duration::from_secs(0),

            // 4) Update REGISTERS (dep 4)
            time_update_registers: Duration::from_secs(0),
            time_evaluate_rd_ops: Duration::from_secs(0),
            time_blind_select_rd: Duration::from_secs(0),
            time_refresh_rd: Duration::from_secs(0),
            time_write_rd: Duration::from_secs(0),

            // 5) Update RAM (dep 4)
            time_update_ram: Duration::from_secs(0),
            time_ram_update_op_eval: Duration::from_secs(0),
            time_blind_select_ram_value: Duration::from_secs(0),
            time_refresh_ram_value: Duration::from_secs(0),
            time_write_ram: Duration::from_secs(0),

            // 6) Update PC (dep 4)
            time_update_pc: Duration::from_secs(0),

            pc_val_fhe_uint_noise: 0.0,
            imm_val_fhe_uint_noise: 0.0,
            ram_addr_read_noise: 0.0,
            ram_val_read_noise: 0.0,
            rd_val_fhe_uint_noise: 0.0,
        }
    }
}

pub fn measure_duration<F>(mut operation: F) -> Duration
where
    F: FnMut(),
{
    let start: Instant = Instant::now();
    operation();
    start.elapsed()
}
