use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;
use teloxide::prelude::ChatId;

#[derive(Debug, Default, Clone)]
pub struct SubsState {
    subscribers: HashMap<i64, usize>,
}

impl SubsState {
    pub fn add(&mut self, uid: i64) {
        self.subscribers.insert(uid, 0);
    }

    pub fn chat_ids(self) -> Vec<ChatId> {
        self.subscribers
            .keys()
            .map(|id| ChatId(id.clone()))
            .collect()
    }
}

impl FromStr for SubsState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let subscribers = s
            .split('\n')
            .map(|l| {
                let l_split = l.split('\t').collect::<Vec<_>>();
                (
                    l_split[0].parse::<i64>().unwrap(),
                    l_split[1].parse::<usize>().unwrap(),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(SubsState { subscribers })
    }
}

impl ToString for SubsState {
    fn to_string(&self) -> String {
        self.subscribers
            .iter()
            .map(|(uid, last_idx)| format!("{}\t{}", uid, last_idx))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
