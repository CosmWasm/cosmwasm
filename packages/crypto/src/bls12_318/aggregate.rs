use super::points::{g1_from_fixed, g2_from_fixed, InvalidPoint, G1, G2};

const G1_POINT_SIZE: usize = 48;
const G2_POINT_SIZE: usize = 96;

/// Takes a list of points in G1 (48 bytes each) and aggregates them.
///
/// This is like Aggregate from <https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-05>
/// but works for signatures as well as public keys.
pub fn bls12_381_aggregate_g1(points: &[u8]) -> Result<[u8; 48], InvalidPoint> {
    if points.len() % G1_POINT_SIZE != 0 {
        return Err(InvalidPoint::DecodingError {});
    }

    let points_count = points.len() / G1_POINT_SIZE;

    use rayon::prelude::*;

    let points: Vec<[u8; 48]> = points
        .chunks_exact(G1_POINT_SIZE)
        .map(|data| {
            let mut buf = [0u8; 48];
            buf[..].clone_from_slice(data);
            buf
        })
        .collect();

    let mut decoded_points = Vec::with_capacity(points_count);
    points
        .par_iter()
        .map(g1_from_fixed)
        .collect_into_vec(&mut decoded_points);

    let out: Result<Vec<G1>, InvalidPoint> = decoded_points.into_iter().collect();
    let out = out?;

    let out = g1_sum(&out);

    Ok(out.to_compressed())
}

/// Takes a list of points in G2 (96 bytes each) and aggregates them.
///
/// This is like Aggregate from <https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-05>
/// but works for signatures as well as public keys.
pub fn bls12_381_aggregate_g2(points: &[u8]) -> Result<[u8; 96], InvalidPoint> {
    if points.len() % G2_POINT_SIZE != 0 {
        return Err(InvalidPoint::DecodingError {});
    }

    let points_count = points.len() / G2_POINT_SIZE;

    use rayon::prelude::*;

    let points: Vec<[u8; 96]> = points
        .chunks_exact(G2_POINT_SIZE)
        .map(|data| {
            let mut buf = [0u8; 96];
            buf[..].clone_from_slice(data);
            buf
        })
        .collect();

    let mut decoded_points = Vec::with_capacity(points_count);
    points
        .par_iter()
        .map(g2_from_fixed)
        .collect_into_vec(&mut decoded_points);

    let out: Result<Vec<G2>, InvalidPoint> = decoded_points.into_iter().collect();
    let out = out?;

    let out = g2_sum(&out);

    Ok(out.to_compressed())
}

/// Creates a sum of points in G1.
///
/// This is fast since math is done on projective points. Parallelization does not help here
/// for ~500 elements.
#[inline]
pub fn g1_sum(elements: &[G1]) -> G1 {
    elements.iter().sum()
}

/// Creates a sum of points in G2.
///
/// This is fast since math is done on projective points. Parallelization does not help here
/// for ~500 elements.
#[inline]
pub fn g2_sum(elements: &[G2]) -> G2 {
    elements.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::super::points::{g1_from_variable, g1s_from_variable};
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use base64_serde::base64_serde_type;
    use hex_literal::hex;

    base64_serde_type!(Base64Standard, STANDARD);

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct EthPubkey(#[serde(with = "Base64Standard")] Vec<u8>);

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct EthHeaders {
        public_keys: Vec<EthPubkey>,
        #[serde(with = "Base64Standard")]
        message: Vec<u8>,
        #[serde(with = "Base64Standard")]
        signature: Vec<u8>,
        #[serde(with = "Base64Standard")]
        aggregate_pubkey: Vec<u8>,
    }

    const ETH_HEADER_FILE: &str =
        include_str!("../../testdata/eth-headers/1699693797.394876721s.json");

    fn read_eth_header_file() -> EthHeaders {
        serde_json::from_str(ETH_HEADER_FILE).unwrap()
    }

    /// Arbitrary point in G1
    fn p1() -> G1 {
        // Public key of classic League of Entropy Mainnet (curl -sS https://drand.cloudflare.com/info)
        g1_from_fixed(&hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31")).unwrap()
    }

    /// Arbitrary point in G2
    fn p2() -> G2 {
        g2_from_fixed(&hex!("b6ed936746e01f8ecf281f020953fbf1f01debd5657c4a383940b020b26507f6076334f91e2366c96e9ab279fb5158090352ea1c5b0c9274504f4f0e7053af24802e51e4568d164fe986834f41e55c8e850ce1f98458c0cfc9ab380b55285a55")).unwrap()
    }

    #[test]
    fn bls12_318_aggregate_g1_works() {
        let sum = bls12_381_aggregate_g1(b"").unwrap();
        assert_eq!(sum, G1::identity().to_compressed());
    }

    #[test]
    fn bls12_318_aggregate_g2_works() {
        let sum = bls12_381_aggregate_g2(b"").unwrap();
        assert_eq!(sum, G2::identity().to_compressed());
    }

    #[test]
    fn g1_sum_works() {
        // no elements
        let sum = g1_sum(&[]);
        assert_eq!(sum, G1::identity());

        // one element
        let sum = g1_sum(&[G1::identity()]);
        assert_eq!(sum, G1::identity());
        let sum = g1_sum(&[p1()]);
        assert_eq!(sum, p1());

        {
            let file = read_eth_header_file();

            let pubkeys: Vec<&[u8]> = file.public_keys.iter().map(|m| m.0.as_slice()).collect();
            let points: Vec<G1> = g1s_from_variable(&pubkeys)
                .into_iter()
                .map(|res| res.unwrap())
                .collect();
            let expected_sum = g1_from_variable(&file.aggregate_pubkey).unwrap();
            let sum = g1_sum(&points);
            assert_eq!(sum, expected_sum);
        }
    }

    #[test]
    fn g2_sum_works() {
        // no elements
        let sum = g2_sum(&[]);
        assert_eq!(sum, G2::identity());

        // single
        let sum = g2_sum(&[p2()]);
        assert_eq!(sum, p2());

        // multiple 1
        let a = g2_from_fixed(&hex!("b6ed936746e01f8ecf281f020953fbf1f01debd5657c4a383940b020b26507f6076334f91e2366c96e9ab279fb5158090352ea1c5b0c9274504f4f0e7053af24802e51e4568d164fe986834f41e55c8e850ce1f98458c0cfc9ab380b55285a55")).unwrap();
        let b = g2_from_fixed(&hex!("b23c46be3a001c63ca711f87a005c200cc550b9429d5f4eb38d74322144f1b63926da3388979e5321012fb1a0526bcd100b5ef5fe72628ce4cd5e904aeaa3279527843fae5ca9ca675f4f51ed8f83bbf7155da9ecc9663100a885d5dc6df96d9")).unwrap();
        let c = g2_from_fixed(&hex!("948a7cb99f76d616c2c564ce9bf4a519f1bea6b0a624a02276443c245854219fabb8d4ce061d255af5330b078d5380681751aa7053da2c98bae898edc218c75f07e24d8802a17cd1f6833b71e58f5eb5b94208b4d0bb3848cecb075ea21be115")).unwrap();
        let expected = g2_from_fixed(&hex!("9683b3e6701f9a4b706709577963110043af78a5b41991b998475a3d3fd62abf35ce03b33908418efc95a058494a8ae504354b9f626231f6b3f3c849dfdeaf5017c4780e2aee1850ceaf4b4d9ce70971a3d2cfcd97b7e5ecf6759f8da5f76d31")).unwrap();
        let sum = g2_sum(&[a.clone(), b.clone(), c.clone()]);
        assert_eq!(sum, expected);
        let sum = g2_sum(&[b.clone(), a.clone(), c.clone()]);
        assert_eq!(sum, expected);
        let sum = g2_sum(&[c.clone(), b.clone(), a.clone()]);
        assert_eq!(sum, expected);

        // multiple 2
        let a = g2_from_fixed(&hex!("882730e5d03f6b42c3abc26d3372625034e1d871b65a8a6b900a56dae22da98abbe1b68f85e49fe7652a55ec3d0591c20767677e33e5cbb1207315c41a9ac03be39c2e7668edc043d6cb1d9fd93033caa8a1c5b0e84bedaeb6c64972503a43eb")).unwrap();
        let b = g2_from_fixed(&hex!("af1390c3c47acdb37131a51216da683c509fce0e954328a59f93aebda7e4ff974ba208d9a4a2a2389f892a9d418d618418dd7f7a6bc7aa0da999a9d3a5b815bc085e14fd001f6a1948768a3f4afefc8b8240dda329f984cb345c6363272ba4fe")).unwrap();
        let c = g2_from_fixed(&hex!("a4efa926610b8bd1c8330c918b7a5e9bf374e53435ef8b7ec186abf62e1b1f65aeaaeb365677ac1d1172a1f5b44b4e6d022c252c58486c0a759fbdc7de15a756acc4d343064035667a594b4c2a6f0b0b421975977f297dba63ee2f63ffe47bb6")).unwrap();
        let expected = g2_from_fixed(&hex!("ad38fc73846583b08d110d16ab1d026c6ea77ac2071e8ae832f56ac0cbcdeb9f5678ba5ce42bd8dce334cc47b5abcba40a58f7f1f80ab304193eb98836cc14d8183ec14cc77de0f80c4ffd49e168927a968b5cdaa4cf46b9805be84ad7efa77b")).unwrap();
        let sum = g2_sum(&[a.clone(), b.clone(), c.clone()]);
        assert_eq!(sum, expected);
        let sum = g2_sum(&[b.clone(), a.clone(), c.clone()]);
        assert_eq!(sum, expected);
        let sum = g2_sum(&[c.clone(), b.clone(), a.clone()]);
        assert_eq!(sum, expected);

        // multiple 3
        let a = g2_from_fixed(&hex!("91347bccf740d859038fcdcaf233eeceb2a436bcaaee9b2aa3bfb70efe29dfb2677562ccbea1c8e061fb9971b0753c240622fab78489ce96768259fc01360346da5b9f579e5da0d941e4c6ba18a0e64906082375394f337fa1af2b7127b0d121")).unwrap();
        let b = g2_from_fixed(&hex!("9674e2228034527f4c083206032b020310face156d4a4685e2fcaec2f6f3665aa635d90347b6ce124eb879266b1e801d185de36a0a289b85e9039662634f2eea1e02e670bc7ab849d006a70b2f93b84597558a05b879c8d445f387a5d5b653df")).unwrap();
        let c = g2_from_fixed(&hex!("ae82747ddeefe4fd64cf9cedb9b04ae3e8a43420cd255e3c7cd06a8d88b7c7f8638543719981c5d16fa3527c468c25f0026704a6951bde891360c7e8d12ddee0559004ccdbe6046b55bae1b257ee97f7cdb955773d7cf29adf3ccbb9975e4eb9")).unwrap();
        let expected = g2_from_fixed(&hex!("9712c3edd73a209c742b8250759db12549b3eaf43b5ca61376d9f30e2747dbcf842d8b2ac0901d2a093713e20284a7670fcf6954e9ab93de991bb9b313e664785a075fc285806fa5224c82bde146561b446ccfc706a64b8579513cfc4ff1d930")).unwrap();
        let sum = g2_sum(&[a.clone(), b.clone(), c.clone()]);
        assert_eq!(sum, expected);
        let sum = g2_sum(&[b.clone(), a.clone(), c.clone()]);
        assert_eq!(sum, expected);
        let sum = g2_sum(&[c.clone(), b.clone(), a.clone()]);
        assert_eq!(sum, expected);

        // infinity
        let inf = g2_from_fixed(&hex!("c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")).unwrap();
        let sum = g2_sum(&[inf.clone()]);
        assert_eq!(sum, inf);
    }
}
