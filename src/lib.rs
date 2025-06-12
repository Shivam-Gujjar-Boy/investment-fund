use solana_program::{
    entrypoint,
};
use processor::process_instruction;

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "PeerFunds",
    project_url: "https://investment-fund-client.vercel.app/",
    contacts: "https://investment-fund-client.vercel.app/",
    policy: "",
    source_code: "https://github.com/Shivam-Gujjar-Boy/investment-fund",
    preferred_languages: "en",
    auditors: ""
}

pub mod instruction;
pub mod processor;
pub mod state;
pub mod errors;
pub mod utils;

entrypoint!(process_instruction);
