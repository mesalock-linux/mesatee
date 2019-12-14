//! # The MesaTEE SDK v0.1.0
//!
//! The MesaTEE SDK is designed for external users who wants to invoke FaaS
//! functions only. If you want to write a FaaS function enclave, please use
//! mesatee_core instead.
//!
//! The MesaTEE SDK is designed for security and trusted computing. So it
//! enforces top-level of user authentication/authorization and encryption.
//! All the communication channels between users and services, services and
//! services are constructed in remote attesatation based mutual authenticated
//! TLS channel. To connect to a MesaTEE Service, the external user need to get
//! a valid TLS certificate signed by a pre-trusted CA. The pre-trusted CA
//! should be hard-coded in `[ra_config]` section of `{PROJECT_ROOT}/build.toml`
//! .
//!
//! ## Code sample using MesaTEE SDK
//!
//! Sample code is located at `{PROJECT_ROOT}/examples/quickstart`.
//!
//! # What can be done by using MesaTEE SDK
//!
//! ## Single Task FaaS
//!
//! By using this MesaTEE SDK, you can run a 'single task' within the following:
//! three steps:
//!
//! 1. Connect to a Task Management Service node (tms), and a Trusted Distributed
//! File System (tdfs)
//! 2. Create a FaaS task
//! 3. Invoke this Faas task with a payload
//!
//! ## Multi-party Task FaaS
//!
//! And you can run a 'multi-party task'. Suppose two participants are Alice
//! and Bob.
//!
//! 0. Alice and Bob consent on each other's `uid`, say `uid1` and `uid2`.
//! 1. Either Alice or Bob connects to a Task Management Service node (tms), and a
//! Trusted Distributed File System (tdfs) with his/her `uid`.
//! 2. Either Alice or Bob invokes `upload_file` to upload his own secret to
//! tdfs. Both Alice and Bob know his/her remote file path.
//! 3. Either Alice or Bob create a task with the FaaS function name, e.g.
//! `psi` for private set intersection, with the remote file path of his/her
//! own data file. Then he/she gets a `task_id` back and tells the other one.
//! 4. The other one approve this task by this `task_id` and his/her own remote
//! file path of his/her own data.
//! 5. Either of Alice and Bob invokes the function.
//! 6. Both Alice and Bob check the status of this task and get the results
//! once the task is finished.
//!
//! # The FaaS functions
//!
//! As of v0.1.0 today, the current implementation includes a All-in-One worker
//! called Function Node Service (fns). Developers may want to work on:
//!
//! 1. Develop new function enclaves.
//!
//! 2. Use MesaPy to dynamically register and execute functions with inputs.
//!
//! 3. Use WebAssembly Interpreter to register and execute functions with
//! inputs.
//!
//! 4. Use an existing MesaTEE system.
//!
//! For (1), one need to use mesatee_core and make the function enclave
//! working like FNS. The current FNS should be the best code sample.
//!
//! For (2), FNS provides an interface to execute a Python script but without
//! dynamic input parameters. We will support it later.
//!
//! For (3), FNS **ONLY** provides the `Spec` WebAssembly machine, not any
//! machine one can use in production. Using WebAssembly interpreter in
//! production requires a well-defined WebAssembly machine specification and
//! wasmi implementation. One should first implement it using Parity's wasmi
//! and then copy the code to the enclave in FNS's style. Then everything
//! should be fine. One should also consider another compiler toolchain to
//! generate such wasm codes, which act as the input of the registration
//! process.
//!
//! For (4), read the MesaTEE SDK carefully.
//!
//! ## Single task function
//!
//! A single task function can take both immediate input string and file ID
//! input. For example, the gbdt function takes file input as its model, and
//! an immediate input string as the data used for inference. Its direct output
//! message stores the inference result.
//!
//! ## Multi-party function
//!
//! Generally speaking, a multi-party function takes at least one file from
//! each participants, and returns results in files as well. The PSI sample
//! function takes one file from each participant, and returns one file to
//! each participant. And it generates one line of immediate output saying
//! "finished".
//!
//! # Trust Model and Management of Measurements
//!
//! We have included a document on the detail of trust model and management
//! of measurements in the project docs. Here is just a brief.
//!
//! Measurements (MRENCLAVE, MRSIGNER) are critical to every Intel SGX project.
//! Remote attestation should include verification of these two measurements
//! before establishing a communication channel. These two measurements are
//! generated by a tool from Intel called `sgx_sign` during compilation.
//!
//! The current MesaTEE project generate a file containing all of the included
//! Intel SGX enclaves, and saved it at `{PROJECT_ROOT}/release/services/enclave_info.toml`.
//!
//! In a classic multi-party computation scenario, every participant **must**
//! agree with these measurements before any collaboration. In MesaTEE, this
//! is enforced by a multi-party, asymmetric cryptography signing process.
//! We call all of these people who must reach consensus on the enclave
//! measurements "**Auditors**".
//!
//! The auditors hold their own private keys and publish their public keys.
//! Once the auditors agree with the measurements, he/she needs to sign the
//! `enclave_info.toml` with his/her private key, and saves the SHA256 digest
//! of `enclave_info.toml`.
//!
//! To launch a MesaTEE task for multi-party computation, the one who invokes
//! the `Mesatee::new` API needs to collect each participant's public key
//! (in DER format), as well as the SHA256 digest, to construct a struct of
//! `MesateeEnclaveInfo`. Then use it in `Mesatee::new` API. The launching
//! process would automatically triggers signature verification and digest
//! verification, and return fail if anything is tampered or incorrect.
//!
//! We want to emphasize that: **MesaTEE cannot force the user to check all
//! those measurements**. Everybody can choose between checking nothing or
//! checking everything, but his/her choice significantly affects the security
//! guarantees of his/her enclaves and communication channel. This fact leads
//! to the following conclusion:
//!
//! **If `MesateeEnclaveInfo` includes nothing about auditors and measurements,
//! nothing would be checked during remote attestation. Then all of the
//! security and safety guarantees are lost.**
//!
//! Please make sure the auditors and measurements are correctly set up in any
//! production scenario.
//!
//! # User Authentication/Authorization
//!
//! MesaTEE framework contains two stage of user authentication and
//! authorization:
//!
//! 1. Transportation Layer Security -- User/service need to present a valid
//! certificate to connect to the MesaTEE Services.
//! 2. Application layer -- User need to present a valid combination of
//! `(user_id, user_token)` to pass the `verify_user` check.
//!
//! ## Transportation Layer Security
//!
//! Stage 1 is strictly enforced for every MesaTEE connection, including
//! the built-in services and the external users. As an external user, one
//! should present his/her client certificate signed by a trusted root CA.
//!
//! ### The Trusted Root CA
//!
//! We generated a prebuilt CA at `{PROJECT_ROOT}/cert/ca.crt`. During the
//! compilation, MesaTEE reads `{PROJECT_ROOT}/build.toml` to get the path
//! of the CA cert and make the cert hard-coded in the MesaTEE components.
//!
//! To generate the root CA, one can use anything equal to the following
//! commands:
//!
//! ```bash
//! $ openssl ecparam -genkey -name prime256v1 -out ca.key
//! $ openssl req -x509 -new -SHA256 -nodes -key ca.key -days 3650 -out ca.crt
//! ```
//!
//! ### User's TLS certificate
//!
//! User needs to pre-generate his/her TLS certificate signed by the above CA.
//! MesaTEE reads `{PROJECT_ROOT}/build.toml` to read its location and makes
//! it hard-coded. Currently MesaTEE only supports 1 client cert. We have plan
//! to support multiple and configurable client certificate interface later.
//!
//! Here are a reference to generate the user's TLS client certificate:
//!
//! ```bash
//! $ openssl ecparam -genkey -name prime256v1 -out client.key
//! $ openssl pkcs8 -topk8 -nocrypt -in client.key -out client.pkcs8
//! $ openssl req -new -SHA256 -key client.key -nodes -out client.csr
//! $ openssl x509 -req -extfile <(printf "subjectAltName=DNS:localhost,DNS:www.example.com") -days 3650 -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt
//! ```
//!
//! ## Application Layer User Authentication/Authorization
//!
//! The MesaTEE depends on an external user authentication system which can
//! verify user by `user_id` + `user_token`. To implement such a user auth
//! system, one should edit
//!
//!  * mesatee_tms/sgx_trusted_lib/src/data_store.rs
//!  * mesatee_tdfs/sgx_trusted_lib/src/data_store.rs
//!
//! to implement the authentication/authorization system. The current framework
//! does not include anything real. Those functions now always return `true`.

#![deny(missing_docs)]

use fns_client::FNSClient;
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::sgx;
pub use mesatee_core::{Error, ErrorKind, Result};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tdfs_external_client::TDFSClient;
use tms_external_client::TMSClient;
pub use tms_external_proto::TaskStatus;

const SGX_HASH_SIZE: usize = 32;

/// `SgxMeasure` stores the value of MRENCLAVE and MRSIGNER.
pub type SgxMeasure = [u8; SGX_HASH_SIZE];

/// `Mesatee` stands for a connection to MesaTEE Service
///
/// To connect to a MesaTEE Service, one should be clear about:
///
/// 0. User's TLS certificate. Currently we do not support a configurable TLS
/// user certificate due to the limitation of current APIs. But we have plan
/// to support it as soon as possible.
/// 1. The credential to connect. In the current design, it is a combination of
/// `(user_id, user_token)`.
/// 2. The description of Task Management Service, which is a string of "ip:port".
/// We do not support any kind of dns name here because we do not have a name
/// resolution mechanism.
/// 3. The description of Trusted Distributed File System.
pub struct Mesatee {
    /// User ID used for application level authentication/authorization
    user_id: String,
    /// One time user token used for application level authentication/
    /// authorization
    user_token: String,
    /// Description of a Task Management Service. One can construct it by calling
    /// `TargetDesc::new`
    tms_desc: TargetDesc,
    /// Description of a Trusted Distributed File System. One can construct it
    /// by calling `TargetDesc::new`
    tdfs_desc: TargetDesc,
    /// The description of Functional Service Node. We cannot use `TargetDesc`
    /// for it because of the limitation of current API design. And it would
    /// be automically initialized by `new`.
    fns_outbound_desc: OutboundDesc,
}

/// `MesateeTask` stands for a task of a FaaS job.
///
/// To create a FaaS task, the following information is required:
///
/// 1. The task ID, identified by a `String`.
/// 2. The name of the desired function, identified by a `String`.
/// 3. The token of the task, which is used for function level authentication.
/// One have valid `task_id` but no `task_token` cannot work on that task.
/// 4. The description of Function Node Service.
/// 5. The information of the task.
pub struct MesateeTask {
    /// The unique ID of a task
    pub task_id: String,
    /// The name of desired function
    pub function_name: String,
    /// The task token issued by Task Management Service
    pub task_token: String,
    /// The description of the Function Node Service, which is always generated
    /// by the Task Management Service.
    pub fns_desc: Option<TargetDesc>,
    /// The information of the task.
    pub task_info: Option<TaskInfo>,
}

/// `Taskinfo` holds the detailed information of a MesaTEE Task
///
/// Taskinfo is dynamically returned from Task Management Service
///
/// 1. The user ID of the task's creator.
/// 2. The status of all of the participants, including their user IDs and
/// the status of whether they approved this task.
/// 3. One of `Created`, `Ready`, `Running`, `Finished`, `Failed`
pub struct TaskInfo {
    /// The user ID of the task's creator.
    pub creator: String,
    /// The status of all of the participants, including their user IDs and
    /// the status of whether they approved this task.
    pub participants: Vec<(String, bool)>,
    /// One of `Created`, `Ready`, `Running`, `Finished`, `Failed`
    pub status: TaskStatus,
}

/// `MesateeEnclaveInfo` holds the detailed information of a MesaTEE Enclave
///
/// Mesatee EnclaveInfo stores the MRSIGNER/MRENCLAVE of an Intel SGX enclave.
/// These measurements are generated by `sgx_sign` tool from Intel and stored
/// at
pub struct MesateeEnclaveInfo {
    /// `enclave_signers` holds a list of "auditor's public key", and
    /// "auditor's copy of sha256 digest of enclave_info.toml".
    enclave_signers: Vec<(Vec<u8>, PathBuf)>,
    /// holds the path of `enclave_info.toml`.
    enclave_info_file_path: PathBuf,
}

impl MesateeEnclaveInfo {
    /// The `load` function loads the measurement info of a enclave from a
    /// local file and returns a `MesateeEnclaveInfo` struct.
    ///
    /// # Arguments
    ///
    /// * `auditors` - holds a `Vec` of `(&str, &str)`, representing a list of
    /// "Auditors" participated in the consensus of MRENCLAVEs and MRSIGNERs.
    /// The tuple `(&str, &str)` means "auditor's public key in DER format",
    ///
    /// * `enclave_info_file_path` - holds the file path of `enclave_info.toml`.
    ///
    /// # Return Value
    ///
    /// * `Ok(MesateeEnclaveInfo)` - if load successfully.
    ///
    /// * `Err(e)` - loading failed with error of `e`.
    pub fn load(auditors: Vec<(&str, &str)>, enclave_info_file_path: &str) -> Result<Self> {
        let mut enclave_signers: Vec<(Vec<u8>, PathBuf)> = vec![];

        for (der, sha) in auditors.iter() {
            let der_content = fs::read(der)?;
            enclave_signers.push((der_content, PathBuf::from_str(sha).expect("infallible")));
        }
        let enclave_info_file_path = PathBuf::from_str(enclave_info_file_path).expect("infallible");
        let enclave_info = MesateeEnclaveInfo {
            enclave_signers,
            enclave_info_file_path,
        };
        Ok(enclave_info)
    }
}

impl Mesatee {
    /// Create an instance for a MesaTEE connection
    ///
    /// # Arguments
    ///
    /// * `enclave_info` - An immutable reference of `MesateeEnclaveInfo` which
    /// holds the value of all auditor's info and the location of `enclave_info
    /// .txt`.
    ///
    /// * `user_id` - An immutable reference of a `String` that holds current
    /// user's ID.
    ///
    /// * `user_token` - An immutable reference of a `String` that holds current
    /// user's access token.
    ///
    /// * `tms_addr` - An immutable reference of a `String` that holds the
    /// connection information of Task Management Service in format of "ip:port".
    ///
    /// * `tdfs_addr` - An immutable reference of a `String` that holds the
    /// connection information of Trusted Distributed File System in format of
    /// "ip:port".
    ///
    /// # Return Value
    ///
    /// Returns an instance of `MesaTEE` if successed.
    ///
    /// # Example
    ///
    /// Example of creating a single FaaS task:
    ///
    /// ```
    /// let tms_addr = "127.0.0.1:5554";
    /// let tdfs_addr = "127.0.0.1:5065";
    /// let mesaTEE = Mesatee::new("uid1", "token1", tms_addr, tdfs_addr).unwrap();
    /// ```
    pub fn new(
        enclave_info: &MesateeEnclaveInfo,
        user_id: &str,
        user_token: &str,
        tms_addr: SocketAddr,
        tdfs_addr: SocketAddr,
    ) -> Result<Self> {
        let mut enclave_signers: Vec<(&[u8], &Path)> = vec![];
        for (der, hash) in enclave_info.enclave_signers.iter() {
            enclave_signers.push((&der, hash.as_path()));
        }
        let enclave_identities = sgx::load_and_verify_enclave_info(
            &enclave_info.enclave_info_file_path,
            &enclave_signers,
        );

        let tms_outbound_desc = OutboundDesc::new(
            *enclave_identities
                .get("tms")
                .ok_or_else(|| Error::from(ErrorKind::MissingValue))?,
        );
        let tms_desc = TargetDesc::new(tms_addr, tms_outbound_desc);

        let tdfs_outbound_desc = OutboundDesc::new(
            *enclave_identities
                .get("tdfs")
                .ok_or_else(|| Error::from(ErrorKind::MissingValue))?,
        );
        let tdfs_desc = TargetDesc::new(tdfs_addr, tdfs_outbound_desc);

        let fns_outbound_desc = OutboundDesc::new(
            *enclave_identities
                .get("fns")
                .ok_or_else(|| Error::from(ErrorKind::MissingValue))?,
        );
        let mesatee = Self {
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
            tms_desc,
            tdfs_desc,
            fns_outbound_desc,
        };
        Ok(mesatee)
    }

    /// Create a MesaTEE Task
    ///
    /// # Arguments
    ///
    /// * `function_name` - An immutable reference of a `String` that holds
    /// the function's name to be crated.
    ///
    /// # Return Value
    ///
    /// Returns a `mesatee::Result<MesateeTask>`, in which:
    ///
    /// * `Ok(task)` where `task` holds the info of created task.
    ///
    /// * `Err(err)` where `err` represents the failure during task creation.
    ///
    /// # Example
    ///
    /// Example of creating a single FaaS task:
    ///
    /// ```
    /// let task = mesatee.create_task("echo").unwrap();
    /// ```
    ///
    /// Here function `echo` honestly "echo" back. It is guaranteed by
    /// the remote attestation based TLS!
    pub fn create_task(&self, function_name: &str) -> Result<MesateeTask> {
        self._create_task(function_name, None, None)
    }

    /// Create a MesaTEE Task with files uploaded on creation
    ///
    /// It takes two arguments: the function name, and the slice of function
    /// ids generated by TDFS.
    ///
    /// One should upload those files before using this API.
    ///
    /// # Arguments
    ///
    /// * `function_name` - An immutable reference of a `String` that holds
    /// the function's name to be crated.
    ///
    /// * `files` - An immutable reference of a slice of file IDs returned by
    /// TDFS.
    ///
    /// # Return Value
    ///
    /// Returns a `mesatee::Result<MesateeTask>`, in which:
    ///
    /// * `Ok(task)` where `task` holds the info of created task.
    ///
    /// * `Err(err)` where `err` represents the failure during task creation.
    pub fn create_task_with_files(
        &self,
        function_name: &str,
        files: &[&str],
    ) -> Result<MesateeTask> {
        self._create_task(function_name, None, Some(files))
    }

    /// Create a MesaTEE Task with known set of collaborators, and files
    ///
    /// It takes three arguments: the function name, the slice of all the
    /// collaborators, and the slice of function ids generated by TDFS.
    ///
    /// One should upload those files before using this API.
    ///
    /// One should be clear about his/her collaborators before using this API.
    ///
    /// * `function_name` - An immutable reference of a `String` that holds
    /// the function's name to be crated.
    ///
    /// # Arguments
    ///
    /// * `collaborator_list` - An immutable reference of a slice of all
    /// the collabortors' user IDs
    ///
    /// * `files` - An immutable reference of a slice of file IDs returned by
    /// TDFS.
    ///
    /// # Return Value
    ///
    /// Returns a `mesatee::Result<MesateeTask>`, in which:
    ///
    /// * `Ok(task)` where `task` holds the info of created task.
    ///
    /// * `Err(err)` where `err` represents the failure during task creation.
    pub fn create_task_with_collaborators(
        &self,
        function_name: &str,
        collaborator_list: &[&str],
        files: &[&str],
    ) -> Result<MesateeTask> {
        self._create_task(function_name, Some(collaborator_list), Some(files))
    }

    fn _create_task(
        &self,
        function_name: &str,
        collaborator_list: Option<&[&str]>,
        files: Option<&[&str]>,
    ) -> Result<MesateeTask> {
        let collaborator_list = collaborator_list.unwrap_or(&[]);
        let files = files.unwrap_or(&[]);

        let mut tms_client = TMSClient::new(&self.tms_desc, &self.user_id, &self.user_token)?;
        let response = tms_client.request_create_task(function_name, collaborator_list, files)?;
        let fns_desc = TargetDesc::new(
            SocketAddr::new(response.ip, response.port),
            self.fns_outbound_desc.clone(),
        );

        Ok(MesateeTask {
            task_id: response.task_id,
            function_name: function_name.to_owned(),
            task_token: response.task_token,
            fns_desc: Some(fns_desc),
            task_info: None,
        })
    }

    /// Get the task information identified by a task id
    ///
    /// This function connects to TMS and query about task info according to
    /// input task ID.
    ///
    /// # Arguments
    ///
    /// `task_id` - An immutable reference to a `String` which holds the ID of
    /// the task.
    ///
    /// # Return Value
    ///
    /// Returns a `mesatee::Result<MesateeTask>`, in which:
    ///
    /// * `Ok(task)` where `task` holds the info of created task.
    ///
    /// * `Err(err)` where `err` represents the failure during task creation.
    pub fn get_task(&self, task_id: &str) -> Result<MesateeTask> {
        let mut tms_client = TMSClient::new(&self.tms_desc, &self.user_id, &self.user_token)?;
        let response = tms_client.request_get_task(task_id)?;
        let task_info = response.task_info;
        let participants = task_info
            .collaborator_list
            .into_iter()
            .map(|c| (c.user_id, c.approved))
            .collect();

        let fns_desc = TargetDesc::new(
            SocketAddr::new(task_info.ip, task_info.port),
            self.fns_outbound_desc.clone(),
        );

        Ok(MesateeTask {
            task_id: task_id.to_owned(),
            function_name: task_info.function_name,
            task_token: task_info.task_token,
            fns_desc: Some(fns_desc),
            task_info: Some(TaskInfo {
                creator: task_info.user_id,
                participants,
                status: task_info.status,
            }),
        })
    }

    /// Get the results of a FaaS task.
    ///
    /// This function connects to the Task Management Service and fetch the results
    /// of a task.
    ///
    /// # Arguments
    ///
    /// `task_id` - An immutable reference to a `String` which holds the ID of
    /// the task
    ///
    /// # Return Value
    ///
    /// Returns a `mesatee::Result<Vec<String>>`, in which:
    ///
    /// * `Ok(v)` where `v` holds the vector of `String`. Each of `String`
    /// indicates the status of corresponding result files's status. If the
    /// invoked function returns three files, this API would return a `Vec`
    /// of three `String`s.
    ///
    /// * `Err(err)` where `err` represents the failure during task creation.
    pub fn get_task_results(&self, task_id: &str) -> Result<Vec<String>> {
        let mut tms_client = TMSClient::new(&self.tms_desc, &self.user_id, &self.user_token)?;
        let response = tms_client.request_get_task(task_id)?;
        let task_info = response.task_info;
        let mut task_result = vec![];
        task_result.extend_from_slice(&task_info.user_private_result_file_id);
        if let Some(id) = task_info.task_result_file_id {
            task_result.push(id)
        };

        Ok(task_result)
    }

    /// One participant approves a task using his/her task ID and files.
    ///
    /// This function connects to Task Management Service and approve a multi-party
    /// computation task with his/her files.
    ///
    /// # Arguments
    ///
    /// `task_id` - An immutable reference to a `String` which holds the ID of
    /// the task
    ///
    /// `files` - An immutable reference to his/her file IDs.
    ///
    /// # Return Value
    ///
    /// Returns a `mesatee::Result<()>`, in which:
    ///
    /// * `Ok(())` means successfully approved the input task.
    ///
    /// * `Err(err)` means MesaTEE encountered `err` during approval.
    pub fn approve_task_with_files(&self, task_id: &str, files: &[&str]) -> Result<()> {
        let mut tms_client = TMSClient::new(&self.tms_desc, &self.user_id, &self.user_token)?;
        let _ = tms_client.request_update_task(task_id, files)?;

        Ok(())
    }

    /// This function returns the content of a result file in a `Vec<u8>`,
    /// identified by its file ID.
    ///
    /// # Arguments
    ///
    /// `file_id` - It holds a reference to the file ID generated by TDFS.
    ///
    /// # Return Value
    ///
    /// One `mesatee_core::Results<Vec<u8>>` in which:
    ///
    /// * `Ok(v)` holds the file content in `v`.
    ///
    /// * `Err(err)` means MesaTEE encountered `err` during get file.
    pub fn get_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let mut client = TDFSClient::new(&self.tdfs_desc, &self.user_id, &self.user_token)?;
        let content = client.read_file(file_id)?;
        Ok(content)
    }

    /// This function helps upload a file to TDFS and get its file ID.
    ///
    /// # Arguments
    ///
    /// `path` - It holds a reference to the file path. Generally it is a
    /// path of local file system.
    ///
    /// # Return Value
    ///
    /// One `mesatee_core::Results<String>` in which:
    ///
    /// * `Ok(s)` holds the file ID in s
    ///
    /// * `Err(err)` means MesaTEE encountered `err` during uploading file.
    pub fn upload_file(&self, path: &str) -> Result<String> {
        let mut tdfs_client = TDFSClient::new(&self.tdfs_desc, &self.user_id, &self.user_token)?;
        Ok(tdfs_client.save_file(path, "")?)
    }
}

impl MesateeTask {
    /// This function starts the computing of this task.
    ///
    /// # Arguments
    ///
    /// No arguments
    ///
    /// # Return Value
    ///
    /// One `mesatee_core::Results<String>` in which:
    ///
    /// * `Ok(s)` holds the output of worker function. Generally, it holds
    /// part of the computating results on design. It varies on workers.
    ///
    /// * `Err(err)` means MesaTEE encountered `err` during function
    /// execution.
    pub fn invoke(&self) -> Result<String> {
        self._invoke(None)
    }

    /// This function starts the computing of this task with a given payload.
    ///
    /// # Arguments
    ///
    /// `payload` holds the **immediate** input arg of the invoked function.
    /// It could be any thing such as a serialized structure, or just a message
    /// . It varies on workers.
    ///
    /// # Return Value
    ///
    /// One `mesatee_core::Results<String>` in which:
    ///
    /// * `Ok(s)` holds the output of worker function. Generally, it holds
    /// part of the computating results on design. It varies on workers.
    ///
    /// * `Err(err)` means MesaTEE encountered `err` during function
    /// execution.
    pub fn invoke_with_payload(&self, payload: &str) -> Result<String> {
        self._invoke(Some(payload))
    }

    fn _invoke(&self, payload: Option<&str>) -> Result<String> {
        let desc = self
            .fns_desc
            .as_ref()
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
        let mut fns_client = FNSClient::new(desc)?;
        let response = fns_client.invoke_task(
            &self.task_id,
            &self.function_name,
            &self.task_token,
            payload,
        )?;
        Ok(response.result)
    }
}
