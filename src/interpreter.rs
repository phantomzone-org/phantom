//use crate::gadget::Gadget;
//use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::address::Address;
use crate::memory::Memory;
use base2k::VecZnx;

pub struct Interpreter {
    pub log_base_rgsw: usize,
    pub counter: Address,
    pub rs1_addresses: Memory,
    pub rs2_addresses: Memory,
    pub imm: Memory,
    pub op_id_register: Memory,
    pub op_it_memory: Memory,
    pub op_id_counter: Memory,
    pub rd: Memory,
    pub register: VecZnx,
    pub memory: Memory,
    pub ret: bool,
}

impl Interpreter {
    /*
    pub fn new(ring: &Ring<u64>, log_base_rgsw: usize, max_counter: usize) -> Self {
        Self {
            log_base_rgsw: log_base_rgsw,
            counter: Address::new(ring, log_base_rgsw, ring.log_n(), max_counter),
            rs1_addresses: Memory::new(ring),
            rs2_addresses: Memory::new(ring),
            imm: Memory::new(ring),
            op_id_register: Memory::new(ring),
            op_it_memory: Memory::new(ring),
            op_id_counter: Memory::new(ring),
            rd: Memory::new(ring),
            register: ring.new_poly(),
            memory: Memory::new(ring),
            ret: false,
        }
    }

    pub fn init(&mut self, ring: &Ring<u64>) {
        self.counter.set(ring, 0);
    }

    pub fn next(&mut self, ring_circuit_bootstrapping: &Ring<u64>, ring: &Ring<u64>) {
        let rs1_address: u64 = self.rs1_addresses.read(ring, &self.counter);
        let rs2_address: u64 = self.rs2_addresses.read(ring, &self.counter);
        let imm: u64 = self.imm.read(ring, &self.counter);
        let op_id_register: u64 = self.op_id_register.read(ring, &self.counter);
        let op_it_memory: u64 = self.op_it_memory.read(ring, &self.counter);
        let op_id_counter: u64 = self.op_id_counter.read(ring, &self.counter);

        let log_gap: usize = 6;
        let log_base_rgsw = self.log_base_rgsw;

        //let circuit_bootstrapper: CircuitBootstrapper = CircuitBootstrapper::new();

        //circuit_bootstrapper.init(ring_circuit_bootstrapping, log_gap, log_base_rgsw);

        let mut buf_acc_0: Poly<u64> = ring_circuit_bootstrapping.new_poly();
        let mut buf_acc_1: Poly<u64> = ring_circuit_bootstrapping.new_poly();
        let mut buf_acc_2: Poly<u64> = ring_circuit_bootstrapping.new_poly();

        /*
        let mut rs1_address_gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base);
        circuit_bootstrapper.circuit_bootstrap(ring_circuit_bootstrapping, rs1_address as usize, &mut buf_acc_0, &mut buf_acc_1, &mut buf_acc_2, &mut rs1_address_gadget);

        let mut rs2_address_gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base);
        circuit_bootstrapper.circuit_bootstrap(ring_circuit_bootstrapping, rs2_address as usize, &mut buf_acc_0, &mut buf_acc_1, &mut buf_acc_2, &mut rs2_address_gadget);

        let mut imm_gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base);
        circuit_bootstrapper.circuit_bootstrap(ring_circuit_bootstrapping, imm as usize, &mut buf_acc_0, &mut buf_acc_1, &mut buf_acc_2, &mut imm_gadget);

        let mut op_id_register_gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base);
        circuit_bootstrapper.circuit_bootstrap(ring_circuit_bootstrapping, op_id_register as usize, &mut buf_acc_0, &mut buf_acc_1, &mut buf_acc_2, &mut op_id_register_gadget);

        let mut op_it_memory_gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base);
        circuit_bootstrapper.circuit_bootstrap(ring_circuit_bootstrapping, op_it_memory as usize, &mut buf_acc_0, &mut buf_acc_1, &mut buf_acc_2, &mut op_it_memory_gadget);

        let mut op_id_counter_gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base);
        circuit_bootstrapper.circuit_bootstrap(ring_circuit_bootstrapping, op_id_counter as usize, &mut buf_acc_0, &mut buf_acc_1, &mut buf_acc_2, &mut op_id_counter_gadget);
         */
        // Retrieves rs1 and rs2
        //let rs1 = self.register.read(ring, Address(Vec::from_raw(&rs1_address_gadget)));

        // Evaluates all OPS
    }
    */
}
