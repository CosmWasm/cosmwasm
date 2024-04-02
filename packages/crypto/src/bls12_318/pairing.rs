use super::points::{Gt, G1, G2};
use bls12_381::G2Prepared;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// Invoke the pairing function over the pair
pub fn bls12_381_pairing(p: &G1, q: &G2) -> Gt {
    Gt(bls12_381::pairing(&p.0, &q.0))
}

/// Compute the sums of the miller loop invocations over a series of point pairs
/// and execute the final exponentiation.
pub fn bls12_381_multi_miller_loop(points: &[(&G1, &G2)]) -> Gt {
    let mut prepared_g2 = Vec::with_capacity(points.len());
    points
        .par_iter()
        .map(|(_g1, g2)| G2Prepared::from(g2.0))
        .collect_into_vec(&mut prepared_g2);

    let mut terms = Vec::with_capacity(points.len());
    let term_iter = points.iter().map(|(g1, _g2)| &g1.0).zip(prepared_g2.iter());
    terms.extend(term_iter);

    Gt(bls12_381::multi_miller_loop(&terms).final_exponentiation())
}

/// Check whether the following condition holds true:
///
/// $$
/// e(p, q) = e(r, s)
/// $$
pub fn bls12_381_pairing_equality(p: &G1, q: &G2, r: &G1, s: &G2) -> bool {
    let p_neg = -p;
    let terms = [(&p_neg, q), (r, s)];
    bls12_381_multi_miller_loop(&terms).is_identity()
}
