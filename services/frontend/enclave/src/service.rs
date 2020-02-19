use anyhow::Result;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternalClient, UserAuthenticateRequest,
};
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CreateTaskRequest, CreateTaskResponse, GetFunctionRequest, GetFunctionResponse,
    GetFusionDataRequest, GetFusionDataResponse, GetOutputFileRequest, GetOutputFileResponse,
    GetTaskRequest, GetTaskResponse, InvokeTaskRequest, InvokeTaskResponse,
    RegisterFunctionRequest, RegisterFunctionResponse, RegisterInputFileRequest,
    RegisterInputFileResponse, RegisterOutputFileRequest, RegisterOutputFileResponse,
    TeaclaveFrontend,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagementClient;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
enum TeaclaveFrontendError {
    #[error("authentication error")]
    AuthenticationError,
    #[error("lock error")]
    LockError,
}

impl From<TeaclaveFrontendError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveFrontendError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_frontend_service, TeaclaveFrontend, TeaclaveFrontendError)]
#[derive(Clone)]
pub(crate) struct TeaclaveFrontendService {
    authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient>>,
    management_client: Arc<Mutex<TeaclaveManagementClient>>,
}

macro_rules! forward_to_management {
    ($service: ident, $request: ident, $func: ident) => {{
        match $service.authenticate(&$request) {
            Ok(true) => (),
            _ => return Err(TeaclaveFrontendError::AuthenticationError.into()),
        }

        let client = $service.management_client.clone();
        let mut client = client
            .lock()
            .map_err(|_| TeaclaveFrontendError::LockError)?;
        client.metadata_mut().clear();
        client.metadata_mut().extend($request.metadata);

        let response = client.$func($request.message);

        client.metadata_mut().clear();
        let response = response?;
        Ok(response)
    }};
}

impl TeaclaveFrontendService {
    pub(crate) fn new(
        authentication_service_endpoint: Endpoint,
        management_service_endpoint: Endpoint,
    ) -> Result<Self> {
        let authentication_channel = authentication_service_endpoint.connect()?;
        let authentication_client = Arc::new(Mutex::new(
            TeaclaveAuthenticationInternalClient::new(authentication_channel)?,
        ));

        let management_channel = management_service_endpoint.connect()?;
        let management_client = Arc::new(Mutex::new(TeaclaveManagementClient::new(
            management_channel,
        )?));

        Ok(Self {
            authentication_client,
            management_client,
        })
    }
}

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn register_input_file(
        &self,
        request: Request<RegisterInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        forward_to_management!(self, request, register_input_file)
    }

    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        forward_to_management!(self, request, register_output_file)
    }

    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        forward_to_management!(self, request, get_output_file)
    }

    fn get_fusion_data(
        &self,
        request: Request<GetFusionDataRequest>,
    ) -> TeaclaveServiceResponseResult<GetFusionDataResponse> {
        forward_to_management!(self, request, get_fusion_data)
    }

    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        forward_to_management!(self, request, register_function)
    }

    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        forward_to_management!(self, request, get_function)
    }

    fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        forward_to_management!(self, request, create_task)
    }

    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        forward_to_management!(self, request, get_task)
    }

    fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<AssignDataResponse> {
        forward_to_management!(self, request, assign_data)
    }

    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        forward_to_management!(self, request, approve_task)
    }

    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        forward_to_management!(self, request, invoke_task)
    }
}

impl TeaclaveFrontendService {
    fn authenticate<T>(&self, request: &Request<T>) -> anyhow::Result<bool> {
        use anyhow::anyhow;
        let id = request
            .metadata
            .get("id")
            .ok_or_else(|| anyhow!("Missing credential"))?;
        let token = request
            .metadata
            .get("token")
            .ok_or_else(|| anyhow!("Missing credential"))?;
        let credential = UserCredential::new(id, token);
        let auth_request = UserAuthenticateRequest { credential };
        let auth_response = self
            .authentication_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock authentication client"))?
            .user_authenticate(auth_request);
        Ok(auth_response?.accept)
    }
}
