use crate::*;

pub struct ContainerParams {
    pub program_id: Pubkey,
    pub realm_pda: Pubkey,
    pub user: Pubkey,
    pub user_account_pda: Pubkey,
    pub twitter_username: String,
    pub tweet_id: twitter_v2::id::NumericId,
}

impl ContainerParams {
    pub fn decode(container_params: &Vec<u8>) -> std::result::Result<Self, SbError> {
        let params = String::from_utf8(container_params.clone()).unwrap();

        let mut program_id: Pubkey = Pubkey::default();
        let mut realm_pda: Pubkey = Pubkey::default();
        let mut user: Pubkey = Pubkey::default();
        let mut user_account_pda: Pubkey = Pubkey::default();
        let mut twitter_username: String = String::default();
        let mut tweet_id: twitter_v2::id::NumericId = twitter_v2::id::NumericId::new(0);

        for env_pair in params.split(',') {
            let pair: Vec<&str> = env_pair.splitn(2, '=').collect();
            if pair.len() == 2 {
                match pair[0] {
                    "PID" => program_id = Pubkey::from_str(pair[1]).unwrap(),
                    "REALM_PDA" => realm_pda = Pubkey::from_str(pair[1]).unwrap(),
                    "USER" => user = Pubkey::from_str(pair[1]).unwrap(),
                    "USER_ACCOUNT_PDA" => user_account_pda = Pubkey::from_str(pair[1]).unwrap(),
                    "TWITTER_USERNAME" => twitter_username = String::from_str(pair[1]).unwrap(),
                    "TWEET_ID" => tweet_id = twitter_v2::id::NumericId::from_str(pair[1]).unwrap(),
                    _ => {}
                }
            }
        }

        if program_id == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "PID cannot be undefined".to_string(),
            ));
        }
        if realm_pda == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "REALM_PDA cannot be undefined".to_string(),
            ));
        }
        if user == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "USER cannot be undefined".to_string(),
            ));
        }
        if user_account_pda == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "USER_ACCOUNT_PDA cannot be undefined".to_string(),
            ));
        }
        if twitter_username == String::default() {
            return Err(SbError::CustomMessage(
                "TWITTER_USERNAME cannot be undefined".to_string(),
            ));
        }
        if tweet_id.as_u64() == 0 {
            return Err(SbError::CustomMessage(
                "TWEET_ID cannot be undefined".to_string(),
            ));
        }

        Ok(Self {
            program_id,
            realm_pda,
            user,
            user_account_pda,
            twitter_username,
            tweet_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_decode() {
        let request_params_string = format!(
            "PID={},REALM_PDA={},USER={},USER_ACCOUNT_PDA={},TWITTER_USERNAME={},TWEET_ID={}",
            anchor_spl::token::ID,
            anchor_spl::token::ID,
            anchor_spl::token::ID,
            anchor_spl::token::ID,
            "elonmusk",
            "1730464228400631814"
        );
        let request_params_bytes = request_params_string.into_bytes();

        let params = ContainerParams::decode(&request_params_bytes).unwrap();

        assert_eq!(params.program_id, anchor_spl::token::ID);
        assert_eq!(params.user, anchor_spl::token::ID);
        assert_eq!(params.realm_pda, anchor_spl::token::ID);
        assert_eq!(params.user_account_pda, anchor_spl::token::ID);
        assert_eq!(params.twitter_username, "elonmusk");
        assert_eq!(
            params.tweet_id,
            twitter_v2::id::NumericId::from_str("1730464228400631814").unwrap()
        );
    }
}
