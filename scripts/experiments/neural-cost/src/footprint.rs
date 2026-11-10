//! `task_info(TASK_VM_INFO)` 的 `phys_footprint`，與 iOS 實驗的 `Shared/Probe.swift` 同一個指標。

/// `struct task_vm_info`，欄位照 `<mach/task_info.h>`；`phys_footprint` 之後的欄位用 `rest` 接，
/// 核心照它支援的版本填（REV3 起 `rest[2]` 是 `ledger_phys_footprint_peak`）。
#[repr(C)]
struct TaskVmInfo {
    virtual_size: u64,
    region_count: i32,
    page_size: i32,
    resident_size: u64,
    resident_size_peak: u64,
    device: u64,
    device_peak: u64,
    internal: u64,
    internal_peak: u64,
    external: u64,
    external_peak: u64,
    reusable: u64,
    reusable_peak: u64,
    purgeable_volatile_pmap: u64,
    purgeable_volatile_resident: u64,
    purgeable_volatile_virtual: u64,
    compressed: u64,
    compressed_peak: u64,
    compressed_lifetime: u64,
    phys_footprint: u64,
    /// REV2：`min_address`、`max_address`；REV3：`ledger_phys_footprint_peak` 起。
    rest: [u64; 64],
}

const TASK_VM_INFO: libc::task_flavor_t = 22;
/// `rest` 裡 `ledger_phys_footprint_peak` 的位置。
const PEAK_INDEX: usize = 2;

#[derive(Clone, Copy)]
pub struct Footprint {
    /// 目前的 `phys_footprint`（位元組）。
    pub now: u64,
    /// 程序到目前為止 footprint 的最高點；核心不給時是 0。
    pub peak: u64,
}

/// 目前程序的 footprint；失敗時都是 0。
pub fn footprint() -> Footprint {
    // SAFETY: 全是整數欄位，全 0 是合法值。
    let mut info: TaskVmInfo = unsafe { std::mem::zeroed() };
    let words = size_of::<TaskVmInfo>() / size_of::<libc::natural_t>();
    let mut count = words as libc::mach_msg_type_number_t;
    // SAFETY: count 是 info 以 natural_t 計的長度，核心最多寫這麼多，並把 count 改成實際寫的長度。
    #[allow(deprecated)] // libc 建議改用 mach2 crate；這裡只要這一個呼叫。
    let ret = unsafe {
        libc::task_info(
            libc::mach_task_self(),
            TASK_VM_INFO,
            (&raw mut info).cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if ret != libc::KERN_SUCCESS {
        return Footprint { now: 0, peak: 0 };
    }
    let rest_start = std::mem::offset_of!(TaskVmInfo, rest) / size_of::<libc::natural_t>();
    // 一個 u64 佔兩個 natural_t。
    let peak_end = rest_start + (PEAK_INDEX + 1) * 2;
    let peak = if count as usize >= peak_end {
        info.rest[PEAK_INDEX]
    } else {
        0
    };
    Footprint {
        now: info.phys_footprint,
        peak,
    }
}

/// malloc 所有 zone 合計還在用的位元組，用來分辨 footprint 的增量是還在用的還是釋放後留著的頁。
pub fn heap_in_use() -> u64 {
    let mut stats = libc::malloc_statistics_t {
        blocks_in_use: 0,
        size_in_use: 0,
        max_size_in_use: 0,
        size_allocated: 0,
    };
    // SAFETY: zone 傳 null 表示所有 zone 的合計；函式只寫 stats。
    unsafe { libc::malloc_zone_statistics(std::ptr::null_mut(), &mut stats) };
    stats.size_in_use as u64
}
