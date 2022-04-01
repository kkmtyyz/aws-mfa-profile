use aws_sdk_sts::output::GetSessionTokenOutput;
use aws_sdk_sts::Client;
use aws_types::SdkConfig;

pub struct Sts {
    client: Client,
}

impl Sts {
    pub fn new(config: &SdkConfig) -> Sts {
        Sts {
            client: Client::new(config),
        }
    }

    pub async fn get_session_token(&self, serial: &str, token: &str) -> GetSessionTokenOutput {
        self.client
            .get_session_token()
            .serial_number(serial)
            .token_code(token)
            .send()
            .await
            .unwrap()
    }
}
