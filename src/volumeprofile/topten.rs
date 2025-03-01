//! Find top ten highest volume assets.
//! The timeline can be dynamic eg. 24 hours,12 hours,1 hour etc..
//!
// curl --request GET \
//   --url 'https://api.dune.com/api/v1/dex/pairs/solana?allow_partial_results=true&columns=token_pair%2C%20one_day_volume' \
//   --header 'X-Dune-Api-Key: EOf8WO9OwsD8PKCS0G2Hus2mv4YM1ARJ'
