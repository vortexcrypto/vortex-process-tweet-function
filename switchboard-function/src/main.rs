use std::str::FromStr;

pub use switchboard_solana::get_ixn_discriminator;
pub use switchboard_solana::prelude::*;
use twitter_v2::authorization::BearerToken;
use twitter_v2::TwitterApi;

mod params;
pub use params::*;
use twitter_v2::query::TweetField;

#[tokio::main(worker_threads = 12)]
async fn main() {
    // First, initialize the runner instance with a freshly generated Gramine keypair
    let runner = FunctionRunner::new_from_cluster(Cluster::Devnet, None).unwrap();

    // parse and validate user provided request params
    let params = ContainerParams::decode(
        &runner
            .function_request_data
            .as_ref()
            .unwrap()
            .container_params,
    )
    .unwrap();

    // Retrieve provided twitter_username BIO
    // Parse BIO and extract wallet address
    // wallet must match provided wallet
    let auth = BearerToken::new("APP_BEARER_TOKEN");
    let twitter_api = TwitterApi::new(auth);

    let fields: Vec<TweetField> = vec![
        TweetField::AuthorId,
        TweetField::ContextAnnotations,
        TweetField::ConversationId,
        TweetField::CreatedAt,
        TweetField::InReplyToUserId,
        TweetField::PublicMetrics,
        TweetField::Source,
        TweetField::Text,
        TweetField::Withheld,
        // TweetField::Attachments,
        // TweetField::Entities,
        // TweetField::Geo,
        // TweetField::Id,
        // TweetField::Lang,
        // TweetField::NonPublicMetrics,
        // TweetField::OrganicMetrics,
        // TweetField::PromotedMetrics,
        // TweetField::PossiblySensitive,
        // TweetField::ReferencedTweets,
        // TweetField::ReplySettings,
    ];

    let tweet = get_tweet_by_id(&twitter_api, params.tweet_id, fields.clone())
        .await
        .unwrap();

    assert_tweet_eligibility(&tweet).unwrap();

    let points = calculate_tweet_points(&tweet).unwrap();

    // IXN DATA:
    // LEN: 12 bytes
    // [0-8]: Anchor Ixn Discriminator
    // [9-17]: Points
    let mut ixn_data = get_ixn_discriminator("process_tweet_settle").to_vec();

    ixn_data.append(&mut points.to_le_bytes().to_vec());

    // ACCOUNTS:
    // 1. Enclave Signer (signer): our Gramine generated keypair
    // 2. User: our user who made the request
    // 3. Realm
    // 4. User Account PDA
    // 5.
    // 6. Switchboard Function
    // 7. Switchboard Function Request
    let settle_ixn = Instruction {
        program_id: params.program_id,
        data: ixn_data,
        accounts: vec![
            AccountMeta::new_readonly(runner.signer, true),
            AccountMeta::new_readonly(params.user, false),
            AccountMeta::new(params.realm_pda, false),
            AccountMeta::new(params.user_account_pda, false),
            AccountMeta::new(params.user_account_pda, false), // TODO
            AccountMeta::new_readonly(runner.function, false),
            AccountMeta::new_readonly(runner.function_request_key.unwrap(), false),
        ],
    };

    // Then, write your own Rust logic and build a Vec of instructions.
    // Should  be under 700 bytes after serialization
    let ixs: Vec<solana_program::instruction::Instruction> = vec![settle_ixn];

    // Finally, emit the signed quote and partially signed transaction to the functionRunner oracle
    // The functionRunner oracle will use the last outputted word to stdout as the serialized result. This is what gets executed on-chain.
    runner.emit(ixs).await.unwrap();
}

pub async fn get_tweet_by_id(
    twitter_api: &TwitterApi<BearerToken>,
    tweet_id: twitter_v2::id::NumericId,
    tweet_fields: Vec<twitter_v2::query::TweetField>,
) -> std::result::Result<twitter_v2::data::Tweet, SbError> {
    if let Some(tweet) = twitter_api
        .get_tweet(tweet_id)
        .tweet_fields(tweet_fields.into_iter())
        .send()
        .await
        .map_err(|e| {
            println!("err getting user: {:?}", e);

            SbError::CustomMessage("err getting user".to_string())
        })?
        .data()
    {
        return Ok(tweet.clone());
    }

    Err(SbError::CustomMessage("tweet not found".to_string()))
}

// Check if the tweet is eligible for rewards
pub fn assert_tweet_eligibility(tweet: &twitter_v2::Tweet) -> std::result::Result<(), SbError> {
    let now_timestamp = chrono::Utc::now().timestamp();

    // Tweet must be at least 4 hours old
    if tweet.created_at.unwrap().unix_timestamp() >= (now_timestamp - 4 * 3_600) {
        return Err(SbError::CustomMessage(
            "tweet must be at least 4h old".to_string(),
        ));
    }

    // Tweet must contains $VTX tag
    if !tweet.text.contains("$VTX") {
        return Err(SbError::CustomMessage(
            "tweet must contain $VTX".to_string(),
        ));
    }

    // Tweet must contains @Vortexcoin tag
    if !tweet.text.contains("@Vortexcoin") {
        return Err(SbError::CustomMessage(
            "tweet must contain @Vortexcoin".to_string(),
        ));
    }

    if tweet.withheld.is_some() {
        return Err(SbError::CustomMessage("tweet is withheld".to_string()));
    }

    Ok(())
}

const LIKE_MULTIPLIER: u64 = 1;
const REPLY_MULTIPLIER: u64 = 1;
const QUOTE_MULTIPLIER: u64 = 1;
const RETWEET_MULTIPLIER: u64 = 1;

pub fn calculate_tweet_points(tweet: &twitter_v2::Tweet) -> std::result::Result<u64, SbError> {
    let metrics = tweet.public_metrics.as_ref().unwrap();

    let like_count = metrics.like_count as u64;
    let reply_count = metrics.reply_count as u64;
    let quote_count = metrics.quote_count.unwrap() as u64;
    let retweet_count = metrics.retweet_count as u64;

    let points = like_count * LIKE_MULTIPLIER
        + reply_count * REPLY_MULTIPLIER
        + quote_count * QUOTE_MULTIPLIER
        + retweet_count * RETWEET_MULTIPLIER;

    Ok(points)
}

#[cfg(test)]
mod tests {
    use twitter_v2::query::TweetField;

    use super::*;

    #[tokio::test]
    pub async fn test_extract_wallet_from_twitter_bio() {
        // Use dev@vortexcrypto.io dev account
        let auth = BearerToken::new("APP_BEARER_TOKEN");
        let twitter_api = TwitterApi::new(auth);

        let fields: Vec<TweetField> = vec![
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
            TweetField::InReplyToUserId,
            TweetField::PublicMetrics,
            TweetField::Source,
            TweetField::Text,
            TweetField::Withheld,
            // TweetField::ContextAnnotations,
            // TweetField::Attachments,
            // TweetField::Entities,
            // TweetField::Geo,
            // TweetField::Id,
            // TweetField::Lang,
            // TweetField::NonPublicMetrics,
            // TweetField::OrganicMetrics,
            // TweetField::PromotedMetrics,
            // TweetField::PossiblySensitive,
            // TweetField::ReferencedTweets,
            // TweetField::ReplySettings,
        ];

        let original_tweet_with_vtx =
            twitter_v2::id::NumericId::from_str("1734080437859787085").unwrap();

        let tweet = get_tweet_by_id(&twitter_api, original_tweet_with_vtx, fields.clone())
            .await
            .unwrap();

        println!("tweet: {:?}", tweet);

        assert_tweet_eligibility(&tweet).unwrap();

        let points = calculate_tweet_points(&tweet).unwrap();

        println!("points: {}", points);
    }
}
