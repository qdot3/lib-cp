use std::{fmt::Debug, ops::Range, usize};

use ops::ops::Min;
use rustc_hash::FxHashMap;
use sparse_table::SparseTable;

#[derive(Debug, Clone)]
pub struct RMQ<T: Ord> {
    data: Box<[T]>,
    /// 各ブロックの最小値からなる Sparse Table
    summary_rmq: SparseTable<Min<T>>,
    block_size: usize,
    /// 出現したブロックレベルの RMQ の一覧
    block_rmq: Box<[BlockRMQ]>,
    /// ブロックから RMQ へのマップ
    block_to_rmq: Box<[usize]>,
}

impl<T: Ord + Copy> RMQ<T> {
    /// # Time Complexity
    ///
    /// *O*(1)
    pub fn query(&self, range: Range<usize>) -> T {
        let (div_s, rem_s) = (range.start / self.block_size, range.start % self.block_size);
        let (div_e, rem_e) = (range.end / self.block_size, range.end % self.block_size);

        // block_size が usize::MAX のとき、div_s = div_e = 0 が成り立つ。
        if div_s == div_e {
            let i = self.block_rmq[self.block_to_rmq[div_s]].position_by_min(rem_s..rem_e);
            return self.data[self.block_size * div_s + i];
        }

        // div_s < div_e より、start に対応するブロックのサイズは block_size である
        let mut min = {
            let i =
                self.block_rmq[self.block_to_rmq[div_s]].position_by_min(rem_s..self.block_size);
            self.data[self.block_size * div_s + i]
        };
        if rem_e != 0 {
            min = min.min({
                let i = self.block_rmq[self.block_to_rmq[div_e]].position_by_min(0..rem_e);
                self.data[self.block_size * div_e + i]
            });
        }
        if let Some(v) = self.summary_rmq.range_query(div_s + 1..div_e) {
            min = min.min(v)
        }

        min
    }
}

impl<T: Ord + Copy> From<Vec<T>> for RMQ<T> {
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    fn from(values: Vec<T>) -> Self {
        // Vec のサイズは高々 usize::MAX/2 なので、オーバーフローしない
        let block_size = if values.len() < 1 << 4 {
            // 要素数が小さい場合、ブロックサイズが 0 になり、 panic してしまう。
            // ただ１つのブロックが存在するようにすればよい
            usize::MAX
        } else {
            values.len().ilog2() as usize >> 2
        };
        let block_num = values.len().div_ceil(block_size);
        let mut block_rmq = Vec::with_capacity(block_num);
        let mut block_to_rmq = Vec::with_capacity(block_num);

        // ブロックごとに Cartesian Tree を計算し、ブロックレベルのRMQを構築する。
        // ブロックごとの最小値を summary に記録する。
        let mut stack = Vec::new();
        let mut cartesian_tree = FxHashMap::default();
        cartesian_tree.reserve(block_num);
        for block in values.chunks(block_size) {
            // block.len() < usize::BIT/2 より、Cartesian tree を usize にエンコードできる
            let mut cartesian_tree_type = 0;
            for &v in block {
                let mut num_pop = 0;
                while stack.pop_if(|u| *u > v).is_some() {
                    num_pop += 1
                }
                stack.push(v);
                cartesian_tree_type <<= num_pop + 1;
                cartesian_tree_type |= 1;
            }
            cartesian_tree_type <<= stack.len();
            stack.clear();

            // Cartesian Tree が既出の場合、ブロックレベルの RMQ は既知
            if let Some(&i) = cartesian_tree.get(&cartesian_tree_type) {
                block_to_rmq.push(i);
                continue;
            }

            // 新しく Cartesian Tree を計算する
            block_to_rmq.push(block_rmq.len());
            cartesian_tree.insert(cartesian_tree_type, block_rmq.len());
            block_rmq.push(BlockRMQ::new(block));
        }

        for (i, block) in values.chunks(block_size).enumerate() {
            let j = block_rmq[block_to_rmq[i]].position_by_min(0..block.len());
            stack.push(block[j]);
        }

        Self {
            data: values.into_boxed_slice(),
            summary_rmq: SparseTable::from(stack),
            block_size,
            block_rmq: block_rmq.into_boxed_slice(),
            block_to_rmq: block_to_rmq.into_boxed_slice(),
        }
    }
}

/// Cartesian Tree に対して RMQ を求めたときに、最小値を与えるインデックスを計算する
#[derive(Debug, Clone)]
struct BlockRMQ {
    pos_rmq: Box<[usize]>,
    block_size: usize,
}

impl BlockRMQ {
    /// # Time Complexity
    ///
    /// *O*(*N*^2)
    fn new<T: Ord>(block: &[T]) -> Self {
        let n = block.len();
        let mut pos_rmq = vec![!0; n * n].into_boxed_slice();
        let mut offset = 0;
        for mut i in 0..n {
            pos_rmq[offset + i] = i;
            for j in i + 1..n {
                if block[j] < block[i] {
                    i = j
                }
                pos_rmq[offset + j] = i
            }
            offset += n
        }

        Self {
            pos_rmq,
            block_size: n,
        }
    }

    /// 最小値の位置を与える。
    ///
    /// # Time Complexity
    ///
    /// *O*(1)
    fn position_by_min(&self, range: Range<usize>) -> usize {
        assert!(!range.is_empty());
        self.pos_rmq[range.start * self.block_size + range.end - 1]
    }
}

#[cfg(test)]
mod tests {
    use rand::{rng, Rng};

    use super::{BlockRMQ, RMQ};

    fn randomized_vec(n: usize) -> Vec<usize> {
        let mut res = Vec::from_iter(0..n);

        let mut rng = rng();
        for i in 0..n {
            let j = rng.random_range(i..n);
            res.swap(i, j);
        }

        res
    }

    #[test]
    fn block_rmq() {
        let n = 1 << 10;
        for _ in 0..20 {
            let data = randomized_vec(n);
            let block_rmq = BlockRMQ::new(&data);
            for i in 0..n {
                let mut k = i;
                for j in i + 1..n {
                    if data[j - 1] < data[k] {
                        k = j - 1
                    }
                    assert_eq!(data[block_rmq.position_by_min(i..j)], data[k])
                }
            }
        }
    }

    #[test]
    fn rmq() {
        let n = 1 << 12;
        for _ in 0..10 {
            let data = randomized_vec(n);
            let rmq = RMQ::from(data.clone());
            for i in 0..n {
                let mut k = i;
                for j in i + 1..n {
                    if data[j - 1] < data[k] {
                        k = j - 1
                    }
                    assert_eq!(rmq.query(i..j), data[k], "{:?} {}", i..j, data[j - 1])
                }
            }
        }
    }
}
