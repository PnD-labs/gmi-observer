use chrono::{DateTime, Timelike};

pub fn convert_chart_timestamp(timestamp: u64) -> u64 {
    // UNIX 타임스탬프를 NaiveDateTime으로 변환
    // println!("\n timestamp = {:?}\n", timestamp);

    let datetime = DateTime::from_timestamp_millis(timestamp as i64).unwrap();
    // println!("\ndata_time = {:?}\n", datetime);
    // 분과 초를 가져옴
    let minutes = datetime.minute();
    let seconds = datetime.second();

    // 5분 단위로 반올림
    let rounded_minutes = if seconds > 0 { minutes + 1 } else { minutes };
    // println!("\n rounded_minutes{:?}", rounded_minutes);
    let rounded_minutes = (rounded_minutes + 4) / 5 * 5;
    // println!("\n rounded_minutes{:?} ", rounded_minutes);

    // 새로운 시간 생성
    let new_datetime = if rounded_minutes >= 60 {
        datetime
            .with_hour((datetime.hour() + 1) % 24)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    } else {
        datetime
            .with_minute(rounded_minutes)
            .unwrap()
            .with_second(0)
            .unwrap()
    };
    // println!("\nnew date time = {:?}\n", new_datetime);
    // NaiveDateTime을 UNIX 타임스탬프로 변환하여 반환
    new_datetime.timestamp() as u64
}
