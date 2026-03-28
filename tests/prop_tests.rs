use std::collections::VecDeque;

use mqtt_to_dawarich::models::{owntracks::OwntracksPayload, persistent_queue::PersistentQueue};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Push(OwntracksPayload),
    Pop,
    Restart,
}

fn arb_payload() -> impl Strategy<Value = OwntracksPayload> {
    (
        any::<u64>(), // id seed
        any::<i64>(), // timestamp
        any::<f64>(), // lat
        any::<f64>(), // lon
        any::<u8>(),  // bs
    )
        .prop_map(|(id, tst, lat, lon, bs)| OwntracksPayload {
            _type: "location".to_string(),
            _id: Some(format!("id-{}", id)),

            lat,
            lon,
            tst,

            acc: None,
            alt: None,
            cog: None,
            vel: None,
            vac: None,

            bs,

            batt: None,
            conn: None,
            created_at: None,

            tid: None,
            m: None,
            tag: None,

            p: None,
            poi: None,
            image: None,
            imagename: None,

            inregions: None,
            inrids: None,

            ssid: None,
            bssid: None,

            topic: None,
        })
}
fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        10 => arb_payload().prop_map(Op::Push),
        7 => Just(Op::Pop),
        1 => Just(Op::Restart),
    ]
}
fn arb_ops() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(arb_op(), 0..1000)
}

proptest! {
    #[test]
fn queue_operations_maintain_consistency(
        ops in arb_ops()
    ) {
        let temp_dir = tempfile::tempdir()?;
        let queue_path = temp_dir.path().join("test.wal");

        let mut queue: PersistentQueue = PersistentQueue::new(queue_path.clone());
        let mut model: VecDeque<OwntracksPayload> = VecDeque::new();

        for op in ops {
            match op {
                Op::Push(x) => {
                    model.push_back(x.clone());
                    queue.push(x);
                    prop_assert_eq!(queue.queue.len(), model.len());
                }
                Op::Pop => {
                    let actual = queue.pop();
                    queue.commit_pop();

                    let expected = model.pop_front();

                    prop_assert_eq!(actual, expected);
                }
                Op::Restart => {
                    drop(queue);

                    queue = PersistentQueue::new(queue_path.clone());

                    let actual = queue.queue.front().cloned();
                    let expected = model.front().cloned();
                    prop_assert_eq!(actual, expected);

                    prop_assert_eq!(queue.queue.len(), model.len());
                }
            }
        }
    }


}
