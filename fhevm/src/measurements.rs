use std::time::{Duration, Instant};

pub struct Measurements {
    pub cycle_measurements: Vec<PerCycleMeasurements>,
}

impl Measurements {
    pub fn new() -> Self {
        Self {
            cycle_measurements: vec![],
        }
    }

    pub fn average_cycle_time(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.total_cycle_time)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_read_instruction_components(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_read_instruction_components)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_read_registers(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_read_registers)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_prepare_imm_rs1_rs2_values(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_prepare_imm_rs1_rs2_values)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_read_ram(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_read_ram)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_update_registers(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_update_registers)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_update_ram(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_update_ram)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_update_pc(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_update_pc)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_evaluate_rd_ops(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_evaluate_rd_ops)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_blind_selection(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_blind_selection)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_compute_rd_address(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_compute_rd_address)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_write_rd(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_write_rd)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_pcu_prepare(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_pcu_prepare)
            .sum::<Duration>();
        total_cycle_time / self.cycle_measurements.len() as u32
    }

    pub fn average_cycle_time_pc_update_bdd(&self) -> Duration {
        let total_cycle_time = self
            .cycle_measurements
            .iter()
            .map(|measurement| measurement.cycle_time_pc_update_bdd)
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

pub struct PerCycleMeasurements {
    // Layer zero
    pub total_cycle_time: Duration,

    // Layer one
    pub cycle_time_read_instruction_components: Duration,
    pub cycle_time_read_registers: Duration,
    pub cycle_time_prepare_imm_rs1_rs2_values: Duration,
    pub cycle_time_read_ram: Duration,
    pub cycle_time_update_registers: Duration,
    pub cycle_time_update_ram: Duration,
    pub cycle_time_update_pc: Duration,

    // Layer two - update_registers
    pub cycle_time_evaluate_rd_ops: Duration,
    pub cycle_time_blind_selection: Duration,
    pub cycle_time_compute_rd_address: Duration,
    pub cycle_time_write_rd: Duration,

    // Layer two - update_pc
    pub cycle_time_pcu_prepare: Duration,
    pub cycle_time_pc_update_bdd: Duration,

    pub pc_val_fhe_uint_noise: f64,
    pub imm_val_fhe_uint_noise: f64,

    pub ram_addr_read_noise: f64,
    pub ram_val_read_noise: f64,

    pub rd_val_fhe_uint_noise: f64,
}

impl PerCycleMeasurements {
    pub fn new() -> Self {
        Self {
            total_cycle_time: Duration::from_secs(0),

            cycle_time_read_instruction_components: Duration::from_secs(0),
            cycle_time_read_registers: Duration::from_secs(0),
            cycle_time_prepare_imm_rs1_rs2_values: Duration::from_secs(0),
            cycle_time_read_ram: Duration::from_secs(0),
            cycle_time_update_registers: Duration::from_secs(0),
            cycle_time_update_ram: Duration::from_secs(0),
            cycle_time_update_pc: Duration::from_secs(0),

            // Layer two - update_registers
            cycle_time_evaluate_rd_ops: Duration::from_secs(0),
            cycle_time_blind_selection: Duration::from_secs(0),
            cycle_time_compute_rd_address: Duration::from_secs(0),
            cycle_time_write_rd: Duration::from_secs(0),

            // Layer two - update_pc
            cycle_time_pcu_prepare: Duration::from_secs(0),
            cycle_time_pc_update_bdd: Duration::from_secs(0),

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
    let start = Instant::now();
    operation();
    start.elapsed()
}
