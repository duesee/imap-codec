#![no_main]

use imap_codec::AuthenticateDataCodec;
use imap_codec_fuzz::impl_decode_target;

impl_decode_target!(AuthenticateDataCodec);
