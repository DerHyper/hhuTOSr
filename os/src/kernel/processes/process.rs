use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::AtomicUsize;
use crate::library::mutex::Mutex;

static PROCESSES: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Process {
    pub id: usize,
    pub name: String
}

impl Process {
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        Process { id: pid, name: String::from(name) }
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
    /*
     * Hier muss Code eingefuegt werden
     */

    Err("Process not found")
}