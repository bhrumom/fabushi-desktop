use crate::protocol::Failure;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PollOption {
    pub id: u32,
    pub text: String,
    pub voter_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum PollKind {
    Regular {
        #[serde(default)]
        multiple_answers: bool,
    },
    Quiz {
        correct_option_id: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        explanation: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Poll {
    pub poll_id: String,
    pub question: String,
    pub options: Vec<PollOption>,
    pub kind: PollKind,
    pub is_closed: bool,
    pub total_voters: u32,
    pub votes: BTreeMap<String, Vec<u32>>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    pub item_id: String,
    pub text: String,
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date_ms: Option<i64>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoList {
    pub list_id: String,
    pub conversation_id: String,
    pub title: String,
    pub items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Default)]
pub struct PollsTodoState {
    pub polls: BTreeMap<String, Poll>,
    pub todo_lists: BTreeMap<String, TodoList>,
    next_poll_id: u32,
    next_list_id: u32,
    next_todo_id: u32,
}

impl PollsTodoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_poll(
        &mut self,
        question: String,
        options: Vec<String>,
        kind: PollKind,
        now_ms: i64,
    ) -> Result<Poll, Failure> {
        if question.trim().is_empty() {
            return Err(Failure::new(
                "INVALID_QUESTION",
                "poll question cannot be empty",
            ));
        }
        if options.len() < 2 || options.len() > 10 {
            return Err(Failure::new(
                "INVALID_OPTION_COUNT",
                "poll must have between 2 and 10 options",
            ));
        }
        self.next_poll_id += 1;
        let poll_id = format!("poll_{}", self.next_poll_id);

        let poll_options = options
            .into_iter()
            .enumerate()
            .map(|(idx, text)| PollOption {
                id: idx as u32,
                text,
                voter_count: 0,
            })
            .collect();

        let poll = Poll {
            poll_id: poll_id.clone(),
            question,
            options: poll_options,
            kind,
            is_closed: false,
            total_voters: 0,
            votes: BTreeMap::new(),
            created_at_ms: now_ms,
        };
        self.polls.insert(poll_id, poll.clone());
        Ok(poll)
    }

    pub fn vote_poll(
        &mut self,
        poll_id: &str,
        voter_id: String,
        option_ids: Vec<u32>,
    ) -> Result<Poll, Failure> {
        let poll = self
            .polls
            .get_mut(poll_id)
            .ok_or_else(|| Failure::new("POLL_NOT_FOUND", format!("poll {poll_id} not found")))?;

        if poll.is_closed {
            return Err(Failure::new("POLL_CLOSED", "poll is closed"));
        }
        if option_ids.is_empty() {
            return Err(Failure::new(
                "EMPTY_SELECTION",
                "must select at least one option",
            ));
        }

        match &poll.kind {
            PollKind::Regular { multiple_answers } => {
                if !multiple_answers && option_ids.len() > 1 {
                    return Err(Failure::new(
                        "MULTIPLE_ANSWERS_NOT_ALLOWED",
                        "multiple answers not allowed",
                    ));
                }
            }
            PollKind::Quiz { .. } => {
                if option_ids.len() > 1 {
                    return Err(Failure::new(
                        "QUIZ_SINGLE_ANSWER_ONLY",
                        "quiz allows only one selection",
                    ));
                }
            }
        }

        for &id in &option_ids {
            if id as usize >= poll.options.len() {
                return Err(Failure::new(
                    "INVALID_OPTION_ID",
                    format!("option {id} out of bounds"),
                ));
            }
        }

        // Retract existing vote if any
        if let Some(prev) = poll.votes.remove(&voter_id) {
            for id in prev {
                if let Some(opt) = poll.options.get_mut(id as usize) {
                    opt.voter_count = opt.voter_count.saturating_sub(1);
                }
            }
            poll.total_voters = poll.total_voters.saturating_sub(1);
        }

        for &id in &option_ids {
            if let Some(opt) = poll.options.get_mut(id as usize) {
                opt.voter_count += 1;
            }
        }
        poll.votes.insert(voter_id, option_ids);
        poll.total_voters += 1;

        Ok(poll.clone())
    }

    pub fn retract_vote(&mut self, poll_id: &str, voter_id: &str) -> Result<Poll, Failure> {
        let poll = self
            .polls
            .get_mut(poll_id)
            .ok_or_else(|| Failure::new("POLL_NOT_FOUND", format!("poll {poll_id} not found")))?;

        if poll.is_closed {
            return Err(Failure::new("POLL_CLOSED", "poll is closed"));
        }
        if let Some(prev) = poll.votes.remove(voter_id) {
            for id in prev {
                if let Some(opt) = poll.options.get_mut(id as usize) {
                    opt.voter_count = opt.voter_count.saturating_sub(1);
                }
            }
            poll.total_voters = poll.total_voters.saturating_sub(1);
        }
        Ok(poll.clone())
    }

    pub fn close_poll(&mut self, poll_id: &str) -> Result<Poll, Failure> {
        let poll = self
            .polls
            .get_mut(poll_id)
            .ok_or_else(|| Failure::new("POLL_NOT_FOUND", format!("poll {poll_id} not found")))?;
        poll.is_closed = true;
        Ok(poll.clone())
    }

    pub fn create_todo_list(
        &mut self,
        conversation_id: String,
        title: String,
    ) -> Result<TodoList, Failure> {
        if title.trim().is_empty() {
            return Err(Failure::new("INVALID_TITLE", "list title cannot be empty"));
        }
        self.next_list_id += 1;
        let list_id = format!("todo_list_{}", self.next_list_id);
        let list = TodoList {
            list_id: list_id.clone(),
            conversation_id,
            title,
            items: Vec::new(),
        };
        self.todo_lists.insert(list_id, list.clone());
        Ok(list)
    }

    pub fn add_todo_item(
        &mut self,
        list_id: &str,
        text: String,
        assignee_id: Option<String>,
        due_date_ms: Option<i64>,
        now_ms: i64,
    ) -> Result<TodoItem, Failure> {
        let list = self
            .todo_lists
            .get_mut(list_id)
            .ok_or_else(|| Failure::new("LIST_NOT_FOUND", format!("list {list_id} not found")))?;

        if text.trim().is_empty() {
            return Err(Failure::new(
                "INVALID_ITEM_TEXT",
                "todo item text cannot be empty",
            ));
        }

        self.next_todo_id += 1;
        let item_id = format!("todo_item_{}", self.next_todo_id);
        let item = TodoItem {
            item_id,
            text,
            completed: false,
            assignee_id,
            due_date_ms,
            created_at_ms: now_ms,
        };
        list.items.push(item.clone());
        Ok(item)
    }

    pub fn toggle_todo_item(
        &mut self,
        list_id: &str,
        item_id: &str,
        completed: bool,
    ) -> Result<TodoItem, Failure> {
        let list = self
            .todo_lists
            .get_mut(list_id)
            .ok_or_else(|| Failure::new("LIST_NOT_FOUND", format!("list {list_id} not found")))?;

        let item = list
            .items
            .iter_mut()
            .find(|it| it.item_id == item_id)
            .ok_or_else(|| Failure::new("ITEM_NOT_FOUND", format!("item {item_id} not found")))?;

        item.completed = completed;
        Ok(item.clone())
    }

    pub fn delete_todo_item(&mut self, list_id: &str, item_id: &str) -> Result<bool, Failure> {
        let list = self
            .todo_lists
            .get_mut(list_id)
            .ok_or_else(|| Failure::new("LIST_NOT_FOUND", format!("list {list_id} not found")))?;

        let init_len = list.items.len();
        list.items.retain(|it| it.item_id != item_id);
        if list.items.len() == init_len {
            return Err(Failure::new(
                "ITEM_NOT_FOUND",
                format!("item {item_id} not found"),
            ));
        }
        Ok(true)
    }

    pub fn handle_route(&mut self, method: &str, args: &Value) -> Result<Value, Failure> {
        if let Some(op) = method.strip_prefix("polls.") {
            match op {
                "create" => {
                    let question = args
                        .get("question")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let options = args
                        .get("options")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_str)
                                .map(ToString::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    let kind: PollKind = args
                        .get("kind")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(PollKind::Regular {
                            multiple_answers: false,
                        });
                    let now_ms = args
                        .get("nowMs")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                    let poll = self.create_poll(question, options, kind, now_ms)?;
                    Ok(json!({ "poll": poll }))
                }
                "vote" => {
                    let poll_id = args
                        .get("pollId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "pollId required"))?;
                    let voter_id = args
                        .get("voterId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "voterId required"))?
                        .to_string();
                    let option_ids = args
                        .get("optionIds")
                        .and_then(Value::as_array)
                        .map(|arr| arr.iter().filter_map(Value::as_u64).map(|v| v as u32).collect())
                        .unwrap_or_default();

                    let poll = self.vote_poll(poll_id, voter_id, option_ids)?;
                    Ok(json!({ "poll": poll }))
                }
                "retractVote" => {
                    let poll_id = args
                        .get("pollId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "pollId required"))?;
                    let voter_id = args
                        .get("voterId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "voterId required"))?;

                    let poll = self.retract_vote(poll_id, voter_id)?;
                    Ok(json!({ "poll": poll }))
                }
                "close" => {
                    let poll_id = args
                        .get("pollId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "pollId required"))?;
                    let poll = self.close_poll(poll_id)?;
                    Ok(json!({ "poll": poll }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_POLL_METHOD",
                    format!("unrecognized poll method {other}"),
                )),
            }
        } else if let Some(op) = method.strip_prefix("todo.") {
            match op {
                "createList" => {
                    let conversation_id = args
                        .get("conversationId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "conversationId required"))?
                        .to_string();
                    let title = args
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let list = self.create_todo_list(conversation_id, title)?;
                    Ok(json!({ "todoList": list }))
                }
                "addItem" => {
                    let list_id = args
                        .get("listId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "listId required"))?;
                    let text = args
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let assignee_id = args
                        .get("assigneeId")
                        .and_then(Value::as_str)
                        .map(ToString::to_string);
                    let due_date_ms = args.get("dueDateMs").and_then(Value::as_i64);
                    let now_ms = args
                        .get("nowMs")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                    let item = self.add_todo_item(list_id, text, assignee_id, due_date_ms, now_ms)?;
                    Ok(json!({ "todoItem": item }))
                }
                "toggleItem" => {
                    let list_id = args
                        .get("listId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "listId required"))?;
                    let item_id = args
                        .get("itemId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "itemId required"))?;
                    let completed = args.get("completed").and_then(Value::as_bool).unwrap_or(true);

                    let item = self.toggle_todo_item(list_id, item_id, completed)?;
                    Ok(json!({ "todoItem": item }))
                }
                "deleteItem" => {
                    let list_id = args
                        .get("listId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "listId required"))?;
                    let item_id = args
                        .get("itemId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "itemId required"))?;

                    let deleted = self.delete_todo_item(list_id, item_id)?;
                    Ok(json!({ "deleted": deleted }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_TODO_METHOD",
                    format!("unrecognized todo method {other}"),
                )),
            }
        } else {
            Err(Failure::new(
                "UNKNOWN_METHOD",
                format!("unrecognized method {method}"),
            ))
        }
    }
}
