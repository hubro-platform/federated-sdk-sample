use hubro_sdk::fed::TrainingData;
use hubro_sdk::mobile::debug_print_line;
use hubro_sdk::mobile::get_current_platform;
use hubro_sdk::mobile::get_health_records;
use hubro_sdk::records::StepsRecord;
use polars::export::num::ToPrimitive;
use polars::prelude::*;

#[unsafe(no_mangle)]
unsafe extern "C" fn fit() -> i32 {
    debug_print_line(&"Fetching last 7 days of steps data...".to_string());

    let now = wasip1::clock_time_get(wasip1::CLOCKID_REALTIME, 1_000).unwrap();
    let unix_time_seconds = now / 1_000_000_000;
    let one_week_seconds = 7 * 24 * 60 * 60;
    let start_ts = (unix_time_seconds as i32) - one_week_seconds;

    // 1. Fetch data from host
    let data = get_health_records::<StepsRecord>(start_ts, unix_time_seconds as i32);
    let record_count = data.len();

    debug_print_line(&format!("Received {} records from Health Connect", record_count));
    if record_count == 0 { return 0; }

    // 2. Build Series directly from the records
    // We slice the first 10 chars of "YYYY-MM-DDTHH..." to get "YYYY-MM-DD"
    let date_series = Series::new("date", data.iter().map(|p| {
        if p.startTime.len() >= 10 { &p.startTime[0..10] } else { "" }
    }).collect::<Vec<&str>>());

    let step_series = Series::new("steps", data.iter().map(|p| p.count).collect::<Vec<u32>>());

    // 3. Construct DataFrame and IMMEDIATELY drop the source 'data'
    let df = DataFrame::new(vec![date_series, step_series]).unwrap();
    drop(data);
    debug_print_line(&"Raw data dropped, starting Polars aggregation...".to_string());

    // 4. Group by the string date ("YYYY-MM-DD")
    let res = df.lazy()
        .group_by([col("date")])
        .agg([col("steps").sum().alias("daily_steps")])
        .select([col("daily_steps").mean()])
        .collect();

    // 5. Extract and return result
    if let Ok(result_df) = res {
        if let Ok(series) = result_df.column("daily_steps") {
            // Perform the cast and store it in a variable so it lives long enough
            let casted_series = series.cast(&DataType::Float64).unwrap();

            // Now we can safely get the f64 ChunkedArray and the first value
            if let Ok(ca) = casted_series.f64() {
                if let Some(val) = ca.get(0) {
                    debug_print_line(&format!("Calculation complete. Mean daily steps: {:.2}", val));
                    return val.ceil() as i32;
                }
            }
        }
    }

    debug_print_line(&"Warning: Calculation failed or returned no data".to_string());
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn fit_training(training_data: TrainingData) -> i32 {
    0
}