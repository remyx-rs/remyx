#[path = "runtime/runtime.rs"]
mod runtime;
#[path = "runtime/tcp.rs"]
mod tcp;
#[path = "runtime/time.rs"]
mod time;

macro_rules! test {
    ($runtime:ty, $threads:literal, $module:ident :: $test:ident) => {
        #[test]
        fn $test() {
            let rt = <$runtime as remyx::runtime::Runtime>::new($threads);
            remyx::runtime::Runtime::block_on(&rt, $crate::$module::$test::<$runtime>(&rt));
        }
    };
}

#[macro_export]
macro_rules! test_suite {
    ($name:ident, $runtime:ty) => {
        mod $name {
            mod runtime {

                test!(
                    $runtime,
                    1,
                    runtime::block_on_returns_completed_future_value
                );
                test!($runtime, 1, runtime::spawn_returns_result);
            }

            mod tcp {

                test!($runtime, 1, tcp::listener_bind_succeeds_on_localhost);
                test!($runtime, 1, tcp::stream_connect_succeeds_to_bound_listener);
                test!($runtime, 1, tcp::stream_read_write_round_trip_succeeds);
            }

            mod time {

                test!(
                    $runtime,
                    1,
                    time::sleep_delays_for_at_least_requested_duration
                );
                test!(
                    $runtime,
                    1,
                    time::timeout_returns_ok_when_future_completes_in_time
                );
                test!(
                    $runtime,
                    1,
                    time::timeout_returns_timed_out_when_future_exceeds_limit
                );
                test!(
                    $runtime,
                    1,
                    time::timeout_with_zero_duration_returns_immediate_future
                );
            }
        }
    };
}

#[cfg(feature = "tokio")]
test_suite!(tokio, remyx::runtime::tokio::Tokio);
