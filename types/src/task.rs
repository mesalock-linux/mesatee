// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::FunctionArguments;
use crate::Storable;
use crate::*;
use anyhow::{anyhow, bail, ensure, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Iter;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct UserID(String);

impl std::convert::From<String> for UserID {
    fn from(uid: String) -> UserID {
        UserID(uid)
    }
}

impl std::convert::From<&str> for UserID {
    fn from(uid: &str) -> UserID {
        UserID(uid.to_string())
    }
}

impl std::convert::From<UserID> for String {
    fn from(user_id: UserID) -> String {
        user_id.to_string()
    }
}

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type UserList = OwnerList;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct OwnerList {
    pub uids: HashSet<UserID>,
}

impl OwnerList {
    pub fn new<T: IntoIterator>(uids: T) -> Self
    where
        <T as IntoIterator>::Item: ToString,
    {
        OwnerList {
            uids: uids.into_iter().map(|x| UserID(x.to_string())).collect(),
        }
    }

    pub fn contains(&self, uid: &UserID) -> bool {
        self.uids.contains(uid)
    }

    pub fn len(&self) -> usize {
        self.uids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, value: UserID) -> bool {
        self.uids.insert(value)
    }

    pub fn union(mut self, other: Self) -> Self {
        for value in other.uids {
            self.uids.insert(value);
        }
        self
    }

    pub fn unions<I>(i: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        i.into_iter().fold(Self::default(), Self::union)
    }
}

impl<T> std::convert::From<Vec<T>> for OwnerList
where
    T: ToString,
{
    fn from(owners: Vec<T>) -> Self {
        OwnerList::new(owners)
    }
}

impl std::convert::From<OwnerList> for Vec<String> {
    fn from(owners: OwnerList) -> Vec<String> {
        owners.into_iter().map(|uid| uid.to_string()).collect()
    }
}

impl IntoIterator for OwnerList {
    type Item = UserID;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.uids.into_iter()
    }
}

#[derive(Debug, Deserialize, Serialize, std::cmp::PartialEq)]
pub enum TaskStatus {
    Created,
    DataAssigned,
    Approved,
    Staged,
    DataPreparing,
    Running,
    Finished,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Created
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OutputsTags {
    inner: HashMap<String, FileAuthTag>,
}

impl OutputsTags {
    pub fn new(hm: HashMap<String, FileAuthTag>) -> Self {
        Self { inner: hm }
    }

    pub fn iter(&self) -> Iter<String, FileAuthTag> {
        self.inner.iter()
    }
}

impl std::convert::TryFrom<HashMap<String, String>> for OutputsTags {
    type Error = anyhow::Error;
    fn try_from(input: HashMap<String, String>) -> Result<Self> {
        let mut ret = HashMap::with_capacity(input.len());
        for (k, v) in input.iter() {
            let tag = FileAuthTag::from_hex(v)?;
            ret.insert(k.to_string(), tag);
        }
        Ok(OutputsTags::new(ret))
    }
}

impl<S: std::default::Default + std::hash::BuildHasher> std::convert::From<OutputsTags>
    for HashMap<String, String, S>
{
    fn from(tags: OutputsTags) -> HashMap<String, String, S> {
        tags.iter()
            .map(|(k, v)| (k.to_string(), v.to_hex()))
            .collect()
    }
}

impl std::iter::FromIterator<(String, FileAuthTag)> for OutputsTags {
    fn from_iter<T: IntoIterator<Item = (String, FileAuthTag)>>(iter: T) -> Self {
        OutputsTags {
            inner: HashMap::from_iter(iter),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskOutputs {
    pub return_value: Vec<u8>,
    pub output_file_hash: OutputsTags,
}

impl TaskOutputs {
    pub fn new(value: impl Into<Vec<u8>>, output_file_hash: HashMap<String, FileAuthTag>) -> Self {
        TaskOutputs {
            return_value: value.into(),
            output_file_hash: OutputsTags::new(output_file_hash),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskFailure {
    pub reason: String,
}

impl TaskFailure {
    pub fn new(reason: impl ToString) -> Self {
        TaskFailure {
            reason: reason.to_string(),
        }
    }
}

impl std::fmt::Display for TaskFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskFailure {}", self.reason)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct ExternalID {
    pub prefix: String,
    pub uuid: Uuid,
}

impl ExternalID {
    pub fn new(prefix: impl ToString, uuid: Uuid) -> Self {
        ExternalID {
            prefix: prefix.to_string(),
            uuid,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl ToString for ExternalID {
    fn to_string(&self) -> String {
        format!("{}-{}", self.prefix, self.uuid.to_string())
    }
}

impl std::convert::From<&ExternalID> for ExternalID {
    fn from(s: &ExternalID) -> ExternalID {
        s.clone()
    }
}

impl std::convert::TryFrom<&str> for ExternalID {
    type Error = anyhow::Error;
    fn try_from(ext_id: &str) -> Result<Self> {
        let pos = ext_id
            .find('-')
            .ok_or_else(|| anyhow!("Invalid external id: {}", ext_id))?;
        let (part0, part1) = ext_id.split_at(pos);
        ensure!(part1.len() > 1, "Invalid external id: {}", ext_id);
        let eid = ExternalID {
            prefix: part0.to_string(),
            uuid: Uuid::parse_str(&part1[1..])?,
        };
        Ok(eid)
    }
}

impl std::convert::TryFrom<String> for ExternalID {
    type Error = anyhow::Error;
    fn try_from(ext_id: String) -> Result<Self> {
        ext_id.as_str().try_into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskResult {
    NotReady,
    Ok(TaskOutputs),
    Err(TaskFailure),
}

#[cfg(test_mode)]
impl TaskResult {
    pub fn unwrap(self) -> TaskOutputs {
        match self {
            TaskResult::Ok(t) => t,
            TaskResult::Err(e) => {
                panic!("called `TaskResult::unwrap()` on an `Err` value: {:?}", &e)
            }
            TaskResult::NotReady => panic!("called `TaskResult::unwrap()` on NotReady case"),
        }
    }
}

impl Default for TaskResult {
    fn default() -> Self {
        TaskResult::NotReady
    }
}

// This is intended for proto::TaskResult field
// Since proto::TaskResult is a wrapper of One-Of keywords,
// it is always converted to an Option<proto::TaskResult>
// when referenced in a request/response structure.
impl<T> std::convert::TryFrom<Option<T>> for TaskResult
where
    T: TryInto<TaskResult, Error = Error>,
{
    type Error = Error;
    fn try_from(option: Option<T>) -> Result<Self> {
        let ret = match option {
            Some(result) => result.try_into()?,
            None => unreachable!(),
        };
        Ok(ret)
    }
}

impl<T, E> std::convert::From<TaskResult> for Option<std::result::Result<T, E>>
where
    T: From<TaskOutputs>,
    E: From<TaskFailure>,
{
    fn from(task_result: TaskResult) -> Option<std::result::Result<T, E>> {
        match task_result {
            TaskResult::Ok(t) => Some(Ok(t.into())),
            TaskResult::Err(e) => Some(Err(e.into())),
            TaskResult::NotReady => None,
        }
    }
}

const TASK_PREFIX: &str = "task";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Task {
    pub task_id: Uuid,
    pub creator: UserID,
    pub function_id: ExternalID,
    pub function_arguments: FunctionArguments,
    pub executor: Executor,
    pub input_owners_map: HashMap<String, OwnerList>,
    pub output_owners_map: HashMap<String, OwnerList>,
    pub function_owner: UserID,
    pub participants: UserList,
    pub approved_users: UserList,
    pub input_map: HashMap<String, ExternalID>,
    pub output_map: HashMap<String, ExternalID>,
    pub result: TaskResult,
    pub status: TaskStatus,
}

impl Storable for Task {
    fn key_prefix() -> &'static str {
        TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}

impl Task {
    pub fn new(
        requester: UserID,
        executor: Executor,
        function: &Function,
        function_arguments: FunctionArguments,
        input_owners_map: HashMap<String, OwnerList>,
        output_owners_map: HashMap<String, OwnerList>,
    ) -> Self {
        let input_owners = UserList::unions(input_owners_map.values().cloned());
        let output_owners = UserList::unions(output_owners_map.values().cloned());
        let mut participants = UserList::unions(vec![input_owners, output_owners]);
        participants.insert(requester.clone());
        if !function.public {
            participants.insert(function.owner.clone());
        }

        Task {
            task_id: Uuid::new_v4(),
            creator: requester,
            executor,
            function_id: function.external_id(),
            function_owner: function.owner.clone(),
            function_arguments,
            input_owners_map,
            output_owners_map,
            participants,
            ..Default::default()
        }
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn check_function_compatibility(&self, function: &Function) -> Result<()> {
        // check arguments
        let function_arguments: HashSet<&String> = function.arguments.iter().collect();
        let provide_args: HashSet<&String> = self.function_arguments.inner().keys().collect();
        ensure!(
            function_arguments == provide_args,
            "function_arguments mismatch"
        );

        // check input
        let input_args: HashSet<String> = function.inputs.iter().map(|f| f.name.clone()).collect();
        let provide_args: HashSet<String> = self.input_owners_map.keys().cloned().collect();
        ensure!(input_args == provide_args, "input keys mismatch");

        // check output
        let output_args: HashSet<String> =
            function.outputs.iter().map(|f| f.name.clone()).collect();
        let provide_args: HashSet<String> = self.output_owners_map.keys().cloned().collect();
        ensure!(output_args == provide_args, "output keys mismatch");

        Ok(())
    }

    pub fn all_data_assigned(&self) -> bool {
        let input_args: HashSet<String> = self.input_owners_map.keys().cloned().collect();
        let assiged_inputs: HashSet<String> = self.input_map.keys().cloned().collect();
        if input_args != assiged_inputs {
            return false;
        }

        let output_args: HashSet<String> = self.output_owners_map.keys().cloned().collect();
        let assiged_outputs: HashSet<String> = self.output_map.keys().cloned().collect();
        if output_args != assiged_outputs {
            return false;
        }

        true
    }

    pub fn all_approved(&self) -> bool {
        self.participants == self.approved_users
    }

    pub fn assign_input(
        &mut self,
        requester: &UserID,
        fname: &str,
        file: &TeaclaveInputFile,
    ) -> Result<()> {
        ensure!(
            file.owner.contains(requester),
            "Assign: requester is not in the owner list. {:?}.",
            file.external_id()
        );

        match self.input_owners_map.get(fname) {
            Some(owner_list) => {
                ensure!(
                    owner_list == &file.owner,
                    "Assign: file ownership mismatch. {:?}",
                    file.external_id()
                );
            }
            None => bail!(
                "Assign: file name not exist in input_owners_map. {:?}",
                fname
            ),
        };

        ensure!(
            self.input_map.get(fname).is_none(),
            "Assign: file already assigned. {:?}",
            fname
        );
        self.input_map.insert(fname.to_owned(), file.external_id());
        Ok(())
    }

    pub fn assign_output(
        &mut self,
        requester: &UserID,
        fname: &str,
        file: &TeaclaveOutputFile,
    ) -> Result<()> {
        ensure!(
            file.owner.contains(requester),
            "Assign: requester is not in the owner list. {:?}.",
            file.external_id()
        );

        match self.output_owners_map.get(fname) {
            Some(owner_list) => {
                ensure!(
                    owner_list == &file.owner,
                    "Assign: file ownership mismatch. {:?}",
                    file.external_id()
                );
            }
            None => bail!(
                "Assign: file name not exist in output_owners_map. {:?}",
                fname
            ),
        };

        ensure!(
            self.output_map.get(fname).is_none(),
            "Assign: file already assigned. {:?}",
            fname
        );
        self.output_map.insert(fname.to_owned(), file.external_id());
        Ok(())
    }
}
