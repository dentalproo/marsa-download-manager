//! How much memory this program holds.
//!
//! Used by the measurement of ROADMAP phase 2 («قياس الأداء: … والذاكرة») in
//! `docs/PERFORMANCE.md`, and by nothing a download needs: reading a counter
//! never changes what is downloaded.
//!
//! The counters are the resident set: the pages that are in memory now
//! ([`rss_bytes`]) and the most it has been at once since the program
//! started ([`peak_rss_bytes`]). Both are `None` when the system call fails,
//! which callers must print as «unknown» rather than as zero.

/// The pages this program has in memory now, in bytes.
pub fn rss_bytes() -> Option<u64> {
    imp::rss_bytes()
}

/// The most pages this program has had in memory at once, in bytes.
pub fn peak_rss_bytes() -> Option<u64> {
    imp::peak_rss_bytes()
}

#[cfg(unix)]
#[allow(unsafe_code)]
mod imp {
    pub fn peak_rss_bytes() -> Option<u64> {
        // SAFETY: getrusage writes the whole struct it is given, for
        // RUSAGE_SELF, and returns non-zero when it does not.
        let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } != 0 {
            return None;
        }
        let raw = u64::try_from(usage.ru_maxrss).ok()?;
        // Linux counts kilobytes in `ru_maxrss`, macOS counts bytes.
        Some(if cfg!(target_os = "linux") {
            raw.saturating_mul(1024)
        } else {
            raw
        })
    }

    pub fn rss_bytes() -> Option<u64> {
        // `/proc/self/statm` is Linux only; elsewhere the peak is the answer.
        #[cfg(target_os = "linux")]
        {
            let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
            let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
            // SAFETY: sysconf reads a system constant; _SC_PAGESIZE takes no
            // pointer and cannot fail here.
            let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
            let page = u64::try_from(page).ok().filter(|p| *p > 0)?;
            Some(pages.saturating_mul(page))
        }
        #[cfg(not(target_os = "linux"))]
        {
            peak_rss_bytes()
        }
    }
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod imp {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    fn counters() -> Option<PROCESS_MEMORY_COUNTERS> {
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        // SAFETY: the handle is this process's pseudo-handle, which is never
        // closed, and the counters struct is sized by the `cb` set above.
        let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
        (ok != 0).then_some(counters)
    }

    pub fn rss_bytes() -> Option<u64> {
        counters().map(|c| c.WorkingSetSize as u64)
    }

    pub fn peak_rss_bytes() -> Option<u64> {
        counters().map(|c| c.PeakWorkingSetSize as u64)
    }
}

#[cfg(not(any(unix, windows)))]
mod imp {
    pub fn rss_bytes() -> Option<u64> {
        None
    }

    pub fn peak_rss_bytes() -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The counters answer with something a program can hold, and the peak
    /// is never below what is held now.
    #[test]
    fn the_counters_answer_and_the_peak_covers_the_present() {
        let now = rss_bytes().expect("no resident set on this system");
        let peak = peak_rss_bytes().expect("no peak resident set on this system");
        assert!(now > 0, "the resident set is {now} bytes");
        assert!(peak >= now, "peak {peak} is below the present {now}");
    }
}
