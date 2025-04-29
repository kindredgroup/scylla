use std::time::Duration;

use chrono::{DateTime, Utc};
use hdrhistogram::Histogram;
pub struct Analyser {
    print_interval: u64,
    histogram: Histogram<u64>,
    pub tx: tokio::sync::mpsc::Sender<u64>,
    rx: tokio::sync::mpsc::Receiver<u64>,
    min_time: Option<DateTime<Utc>>,
    max_time: Option<DateTime<Utc>>
}

impl Analyser {
    pub fn new(print_interval: u64) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(3000);
        Self {
            print_interval,
            histogram: Histogram::new(2).expect("Unable to initiate histogram"),
            tx,
            rx,
            min_time: None,
            max_time: None
        }
    }

    fn calculate_rate(&self, num_of_samples: f64, max_time: DateTime<Utc>, min_time: DateTime<Utc>) -> u64 {
        let rate =  (num_of_samples/ (max_time.timestamp_millis() as f64 - min_time.timestamp_millis() as f64)) * 1000 as f64;
        rate.round() as u64
    }

    pub fn print_stats(&self) {
        if !self.histogram.is_empty() {
            println!("| P0 | P50 | P90 | P95 | P98 | P99 | P99.9 | count | rate |");
            println!("|----|-----|-----|-----|-----|-----|-------|------|-------|");
            print!("| {} ", self.histogram.value_at_quantile(0.0));
            print!("| {} ", self.histogram.value_at_quantile(0.50));
            print!("| {} ", self.histogram.value_at_quantile(0.90));
            print!("| {} ", self.histogram.value_at_quantile(0.95));
            print!("| {} ", self.histogram.value_at_quantile(0.98));
            print!("| {} ", self.histogram.value_at_quantile(0.99));
            print!("| {} ", self.histogram.value_at_quantile(0.999));
            print!("| {} ", self.histogram.len());
            if self.min_time.is_some() && self.max_time.is_some() {
                let rate = self.calculate_rate(self.histogram.len() as f64, self.max_time.unwrap(), self.min_time.unwrap());
                print!("| {} |", rate);
            } else {
                print!("| NA |");
            }
            println!(" ");
        }
    }



    pub async fn start(&mut self) {
        loop {
            match self.rx.recv().await {
                Some(val) => {
                    self.histogram.record(val);
                    if self.min_time.is_none() {
                        self.min_time = Some(Utc::now())
                    } else {
                        self.max_time = Some(Utc::now())
                    }
                },
                None => {break;}
            }
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.print_interval));
        loop{
            tokio::select! {
                value = self.rx.recv() => {
                    if let Some(val) = value {
                        self.histogram.record(val);
                        if self.min_time.is_none() {
                            self.min_time = Some(Utc::now())
                        } else {
                            self.max_time = Some(Utc::now())
                        }
                    }
                }
                _ = interval.tick() => {
                    self.print_stats();
                }
            }
        }
        
    }
}