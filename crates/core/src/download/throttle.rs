//! DL-008: Bandwidth throttling.
//!
//! Limits download speed to a configured maximum across all active threads.
//! Each worker gets a [`ThrottleHandle`] that sleeps after block writes
//! if the speed exceeds the per-thread limit.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Shared throttle state across all download workers.
#[derive(Debug)]
pub struct Throttle {
	/// Global speed limit in bytes per second. 0 = unlimited.
	max_bytes_per_sec: u64,
	/// Number of currently active download threads.
	active_threads: AtomicU32,
}

impl Throttle {
	/// Create a new throttle with the given limit in KB/s.
	/// `max_speed_kbps = 0` means unlimited (no throttling).
	pub fn new(max_speed_kbps: u32) -> Arc<Self> {
		Arc::new(Self {
			max_bytes_per_sec: max_speed_kbps as u64 * 1024,
			active_threads: AtomicU32::new(0),
		})
	}

	/// Whether throttling is active (limit > 0).
	pub fn is_active(&self) -> bool {
		self.max_bytes_per_sec > 0
	}

	/// BR-DL-018: Register a new active thread. Returns updated count.
	pub fn thread_started(&self) -> u32 {
		self.active_threads.fetch_add(1, Ordering::Relaxed) + 1
	}

	/// BR-DL-018: Unregister a finished thread. Returns updated count.
	pub fn thread_finished(&self) -> u32 {
		self.active_threads.fetch_sub(1, Ordering::Relaxed) - 1
	}

	/// Current per-thread speed limit in bytes per second.
	pub fn per_thread_limit(&self) -> u64 {
		if self.max_bytes_per_sec == 0 {
			return 0; // unlimited
		}
		let threads = self.active_threads.load(Ordering::Relaxed).max(1) as u64;
		self.max_bytes_per_sec / threads
	}

	/// Create a handle for one worker thread.
	pub fn handle(self: &Arc<Self>) -> ThrottleHandle {
		ThrottleHandle {
			throttle: self.clone(),
			interval_start: Instant::now(),
			interval_bytes: 0,
		}
	}
}

/// Per-worker throttle handle. Tracks bytes written in the current interval.
pub struct ThrottleHandle {
	throttle: Arc<Throttle>,
	interval_start: Instant,
	interval_bytes: u64,
}

impl ThrottleHandle {
	/// BR-DL-017: Called after each block write. Sleeps if speed exceeds limit.
	///
	/// Returns the sleep duration (0 if no throttling needed).
	pub async fn on_bytes_written(&mut self, bytes: u64) -> Duration {
		if !self.throttle.is_active() {
			return Duration::ZERO;
		}

		self.interval_bytes += bytes;

		let limit = self.throttle.per_thread_limit();
		if limit == 0 {
			return Duration::ZERO;
		}

		let elapsed = self.interval_start.elapsed();
		let expected_duration = Duration::from_secs_f64(self.interval_bytes as f64 / limit as f64);

		if expected_duration > elapsed {
			let sleep_time = expected_duration - elapsed;
			tokio::time::sleep(sleep_time).await;

			// Reset interval after sleeping to avoid drift accumulation
			self.interval_start = Instant::now();
			self.interval_bytes = 0;

			sleep_time
		} else {
			// Reset interval periodically (every ~1s) to keep calculations fresh
			if elapsed > Duration::from_secs(1) {
				self.interval_start = Instant::now();
				self.interval_bytes = 0;
			}
			Duration::ZERO
		}
	}

	/// Register this worker as active.
	pub fn start(&self) -> u32 {
		self.throttle.thread_started()
	}

	/// Unregister this worker.
	pub fn finish(&self) -> u32 {
		self.throttle.thread_finished()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// DL-008 | BR-DL-019: max_speed_kbps=0 means unlimited.
	#[test]
	fn dl008_zero_means_unlimited() {
		let throttle = Throttle::new(0);
		assert!(!throttle.is_active());
		assert_eq!(throttle.per_thread_limit(), 0);
	}

	/// DL-008 | Main Success (Step 1): Limit is divided across threads.
	#[test]
	fn dl008_per_thread_limit() {
		let throttle = Throttle::new(1000); // 1000 KB/s = 1_024_000 B/s
		assert!(throttle.is_active());

		throttle.thread_started();
		throttle.thread_started();
		// 2 threads: 1_024_000 / 2 = 512_000
		assert_eq!(throttle.per_thread_limit(), 512_000);
	}

	/// DL-008 | A1: Single thread gets full bandwidth.
	#[test]
	fn dl008_single_thread_full_bandwidth() {
		let throttle = Throttle::new(1000); // 1_024_000 B/s
		throttle.thread_started();
		assert_eq!(throttle.per_thread_limit(), 1_024_000);
	}

	/// DL-008 | BR-DL-018: Dynamic adjustment when threads change.
	#[test]
	fn dl008_dynamic_thread_adjustment() {
		let throttle = Throttle::new(1000); // 1_024_000 B/s

		throttle.thread_started(); // 1 thread
		assert_eq!(throttle.per_thread_limit(), 1_024_000);

		throttle.thread_started(); // 2 threads
		assert_eq!(throttle.per_thread_limit(), 512_000);

		throttle.thread_started(); // 3 threads
		assert_eq!(throttle.per_thread_limit(), 341_333); // 1_024_000 / 3

		throttle.thread_finished(); // back to 2
		assert_eq!(throttle.per_thread_limit(), 512_000);
	}

	/// DL-008 | BR-DL-018: Zero active threads → use 1 (avoid divide by zero).
	#[test]
	fn dl008_zero_threads_no_panic() {
		let throttle = Throttle::new(1000);
		// No threads registered — per_thread_limit uses max(1)
		assert_eq!(throttle.per_thread_limit(), 1_024_000);
	}

	/// DL-008 | BR-DL-017: Negligible sleep for tiny write at high limit.
	#[tokio::test]
	async fn dl008_negligible_sleep_under_limit() {
		let throttle = Throttle::new(10_000); // 10 MB/s
		throttle.thread_started();
		let mut handle = throttle.handle();

		// Write a tiny amount — may sleep microseconds, but not meaningfully
		let slept = handle.on_bytes_written(1024).await;
		assert!(slept < Duration::from_millis(10), "expected <10ms, got {:?}", slept);
	}

	/// DL-008 | BR-DL-017: Throttle sleeps when over limit.
	#[tokio::test]
	async fn dl008_sleeps_when_over_limit() {
		let throttle = Throttle::new(1); // 1 KB/s = 1024 B/s
		throttle.thread_started();
		let mut handle = throttle.handle();

		// Write 2KB at 1KB/s limit → should need ~1s sleep
		let start = Instant::now();
		let slept = handle.on_bytes_written(2048).await;

		assert!(slept > Duration::from_millis(500), "should have slept >500ms, got {:?}", slept);
		assert!(start.elapsed() > Duration::from_millis(500));
	}

	/// DL-008 | BR-DL-019: Unlimited means no sleep even for large writes.
	#[tokio::test]
	async fn dl008_unlimited_no_sleep() {
		let throttle = Throttle::new(0); // unlimited
		throttle.thread_started();
		let mut handle = throttle.handle();

		let slept = handle.on_bytes_written(100_000_000).await;
		assert_eq!(slept, Duration::ZERO);
	}

	/// DL-008 | Handle: start/finish track thread count.
	#[test]
	fn dl008_handle_start_finish() {
		let throttle = Throttle::new(1000);
		let handle = throttle.handle();

		assert_eq!(handle.start(), 1);
		assert_eq!(handle.start(), 2); // calling start twice = 2 threads
		assert_eq!(handle.finish(), 1);
		assert_eq!(handle.finish(), 0);
	}
}
