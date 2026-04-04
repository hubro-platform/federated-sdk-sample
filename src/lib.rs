use hubro_sdk::fed::TrainingData;
use hubro_sdk::mobile::debug_print_line;
use hubro_sdk::mobile::get_current_platform;
use hubro_sdk::mobile::get_health_records;
use hubro_sdk::records::StepsRecord;
use polars::export::num::ToPrimitive;
use polars::prelude::*;

#[unsafe(no_mangle)]
unsafe extern "C" fn fit() -> i32 {
    debug_print_line(&"Fetching last 7 days of steps data".to_string());
    let now = wasip1::clock_time_get(wasip1::CLOCKID_REALTIME, 1_000).unwrap();
    let unix_time_seconds = now / 1_000_000_000;
    let one_week_seconds = 7 * 24 * 60 * 60;
    let start_time = (unix_time_seconds as i32) - one_week_seconds;
    let data = get_health_records::<StepsRecord>(start_time, unix_time_seconds as i32);

    if (get_current_platform() == hubro_sdk::mobile::Platform::Android) {
        debug_print_line(&format!("Received {} Health Connect records", data.len()));
    } else {
        debug_print_line(&format!("Received {} HealthKit records", data.len()));
    }

    let start_time: Vec<String> = data.iter().map(|p| p.startTime.clone()).collect();
    let end_time: Vec<String> = data.iter().map(|p| p.endTime.clone()).collect();
    let steps: Vec<u32> = data.iter().map(|p| p.count.clone()).collect();

    let options = StrptimeOptions {
        format: Some("%Y-%m-%dT%H:%M:%S.%fZ".into()),
        ..Default::default()
    };

    let dataframe = df! {
        "start_time" => start_time,
        "end_time" => end_time,
        "steps" => steps,
    }
    .unwrap();

    let lazy_df = dataframe
        .lazy()
        .with_column(
            col("start_time")
                .str()
                .strptime(DataType::Date, options.clone(), lit(false)) // parse to Date
                .alias("date"),
        )
        .group_by([col("date")])
        .agg([col("steps").sum().alias("daily_steps")])
        .select([col("daily_steps").mean().alias("mean_daily_steps")])
        .collect()
        .unwrap();
    //
    if let Ok(series) = lazy_df.column("mean_daily_steps") {
        if let Some(steps_val) = series.f64().unwrap().get(0) {
            debug_print_line(&format!("Mean daily steps: {}", steps_val));
            return steps_val.ceil().to_i32().unwrap().try_into().unwrap();
        }
    }

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn fit_training(training_data: TrainingData) -> i32 {
    0
}

// #[no_mangle]
// pub extern "C" fn addx(left: i32, right: i32) -> i32 {
//     3
// let dataset = linfa_datasets::diabetes();
//
// let lin_reg = TweedieRegressor::params().power(0.).alpha(0.);
// let model = lin_reg.fit(&dataset).unwrap();
// let s3 = unsafe { push_result(4.5) };
// let s4 = unsafe { str_test("fds") };
//     3
// }
