use std::collections::VecDeque;

pub trait Rng<T> {
    fn generate(&mut self, range: impl std::ops::RangeBounds<T>) -> T;
}

pub fn permutations<T: Clone>(
    k: usize,
    deck: Vec<T>,
    mut rng: impl Rng<usize>,
) -> impl Iterator<Item = Vec<T>> {
    // Robert Floyd's Algorithm: sample a single random permutation
    // https://dl.acm.org/doi/pdf/10.1145/30401.315746
    //
    // initialize sequence S to empty
    // for J := N - M + 1 to N do
    //   T := RandInt(1, J)
    //   if T is not in S then
    //     prefix T to S
    //   else
    //     insert J in S after T
    let mut result = VecDeque::with_capacity(k);
    let n = deck.len();
    std::iter::from_fn(move || {
        result.clear();
        for j in (n - k)..n {
            let t = rng.generate(0..=j);
            if let Some(i) = result.iter().position(|x| *x == t) {
                result.insert(i + 1, j);
            } else {
                result.push_front(t);
            }
        }
        assert_eq!(k, result.len());
        Some(result.iter().map(|i| deck[*i].clone()).collect())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastrand::Rng;
    use itertools::Itertools;

    fn mean(data: &[usize]) -> f64 {
        // rust cookbook
        let sum = data.iter().sum::<usize>() as f64;
        let count = data.len() as f64;
        sum / count
    }

    fn std_deviation(data: &[usize]) -> f64 {
        // rust cookbook
        let data_mean = mean(data);

        let variance = data
            .iter()
            .map(|value| {
                let diff = data_mean - (*value as f64);

                diff * diff
            })
            .sum::<f64>()
            / data.len() as f64;

        variance.sqrt()
    }

    #[test]
    fn test_permutations_are_uniformly_distributed() {
        let n = 1_000_000;
        let threshold = 1.0 / 250.0;

        for seed in [0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144] {
            let vec = vec!["a", "b", "c"];
            let rng = Rng::with_seed(seed);
            let result = permutations(2, vec, rng)
                .take(n)
                .map(|combo| combo.join(","))
                .counts()
                .values()
                .copied()
                .collect_vec();
            let std_dev = std_deviation(&result);
            let mean = mean(&result);
            assert_eq!(6, result.len());
            assert!(
                std_dev < threshold * mean as f64,
                "std deviation of {:.3}% was more than {:.3}%",
                100.0 * (std_dev / mean),
                threshold * 100.0
            );
        }
    }

    #[test]
    fn test_permutations_n_3_k_2() {
        let vec = vec!["a", "b", "c"];
        let rng = Rng::with_seed(1);
        let result = permutations(2, vec, rng)
            .take(100)
            .map(|combo| {
                assert_eq!(2, combo.len());
                combo.join(",")
            })
            .counts()
            .keys()
            .cloned()
            .sorted()
            .collect_vec();
        assert_eq!(6, result.len());
        assert_eq!(vec!["a,b", "a,c", "b,a", "b,c", "c,a", "c,b"], result);
    }
}
