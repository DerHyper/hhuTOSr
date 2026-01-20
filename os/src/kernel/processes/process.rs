use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::{self, Vec};
use core::sync::atomic::AtomicUsize;
use crate::consts::{PAGE_SIZE, STACK_SIZE, USER_CODE_VIRT_START, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::processes::vma::{self, VMA};
use crate::library::mutex::Mutex;

static PROCESSES: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Process {
    pub id: usize,
    pub name: String,
    pub vmas: Vec<VMA>
}

impl Process {
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        
        // VMAs
        let mut vmas = Vec::new();

        Process { id: pid, name: String::from(name), vmas }
    }

    pub fn any_vma_overlaps(&self, other: &VMA) -> bool {
        for vma in self.vmas.iter() {
            if vma.overlaps(other) {
                return true;
            }
        }
        return false;
    }

    pub fn add_vma(&mut self, vma: VMA) -> Result<(), &'static str> {
        if self.any_vma_overlaps(&vma) {
            return Err("VMA is overlapping with existing VMAs in Process");
        }

        self.vmas.push(vma);
        return Ok(());
    }

    pub fn init_vmas(&mut self, vma_code_end_address: u64) {
        let vma_code_start = USER_CODE_VIRT_START as u64;
        let vma_code = VMA::new( vma_code_start, vma_code_end_address, vma::VmaType::Code);
        self.vmas.push(vma_code);

        let vma_stack_start = (USER_STACK_VIRT_END - PAGE_SIZE) as u64;
        let vma_stack_end = USER_STACK_VIRT_END as u64;
        let vma_stack = VMA::new(vma_stack_start, vma_stack_end, vma::VmaType::Stack);
        self.vmas.push(vma_stack);
    }

    fn dump_vmas(&self) {
        kprintln!("VMAs of process '{}'",self.name);

        for vma in self.vmas.iter() {
            kprintln!("   type: {}, start: {:#x}, end: {:#x}", vma.typ, vma.start, vma.end)
        }
    }
}

pub fn add_process(process: Process) {
    PROCESSES.lock().insert(process.id, process);
}

pub fn remove_process(process_id: usize) {
    PROCESSES.lock().remove(&process_id);
}

pub fn get_app_name(process_id: usize) -> Option<String> {
    let processes = PROCESSES.lock();
    let process = processes.get(&process_id);
    if process.is_none() {
        return None;
    }
    return Some(process.unwrap().name.clone());

}

pub fn add_vma(process_id: usize, vma: VMA) -> Result<(), &'static str> {

    if let Some(process) = PROCESSES.lock().get_mut(&process_id){
        return process.add_vma(vma);
    } else {
        return Err("Process not found");   
    }
}

pub fn init_vmas(process_id: usize, vma_code_end_address: u64) {
    let mut processes = PROCESSES.lock();
    let process = processes.get_mut(&process_id).expect("Found no process to init vmas");

    process.init_vmas(vma_code_end_address);
}

pub fn dump_vmas(process_id: usize) {
    let processes = PROCESSES.lock();
    let process = processes.get(&process_id).expect("Found no process to dump vmas");

    process.dump_vmas();
}