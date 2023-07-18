#![no_main]

use imap_codec::imap_types::auth::AuthenticateData;
use imap_codec_fuzz::impl_decode_target;

impl_decode_target!(AuthenticateData);
