use core::{
    backend::{Backend, Module, FFT64},
    glwe::ciphertext::{GLWECiphertextToMut, GLWECiphertextToRef},
    GGSWCiphertext, GLWECiphertext, GLWEOps, GLWEPlaintext, Infos, Scratch, SetMetaData,
};
use std::usize;

use itertools::izip;

pub mod add;

// scratch space required for addition circuit

fn cmux<G: GLWECiphertextToRef>(
    t: &G,
    f: &G,
    s: &GGSWCiphertext<Vec<u8>, FFT64>,
    o: &mut GLWECiphertext<Vec<u8>>,
    module: &Module<FFT64>,
    scratch: &mut Scratch,
) {
    o.sub(module, t, f);
    o.external_product_inplace(module, s, scratch);
    o.add_inplace(module, f);
}
