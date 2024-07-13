#![no_main]

use imap_codec::fragmentizer::{Fragmentizer, MaxMessageSize};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (MaxMessageSize, Vec<Vec<u8>>)| {
    let (mms, data) = input;

    #[cfg(feature = "debug")]
    println!("input: mms={mms:?}, data={data:?}");

    // Ensure `Fragmentizer` recreates input data (when smaller than `mms`).
    let mut emitted_bytes = Vec::new();
    let data_flat = data.iter().flatten().copied().collect::<Vec<u8>>();

    let mut fragmentizer = Fragmentizer::new(mms);

    for chunk in &data {
        loop {
            match fragmentizer.progress() {
                Some(fragment_info) => {
                    #[cfg(feature = "debug")]
                    println!("{fragment_info:?}");

                    // Collect emitted bytes.
                    emitted_bytes.extend_from_slice(fragmentizer.fragment_bytes(fragment_info));
                }
                None => {
                    fragmentizer.enqueue_bytes(chunk);
                    break;
                }
            }
        }
    }

    assert!(emitted_bytes.len() <= data_flat.len());

    let check_prefix = match mms {
        MaxMessageSize::Unlimited => true,
        MaxMessageSize::Limited(limit) => data_flat.len() <= limit as usize,
    };

    if check_prefix {
        #[cfg(feature = "debug")]
        println!("Checking prefix...");

        // Ensure emitted bytes are prefix of fuzzing input data.
        assert_eq!(emitted_bytes, data_flat[..emitted_bytes.len()]);
    }
});
