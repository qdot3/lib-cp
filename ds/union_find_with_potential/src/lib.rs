use ops::Group;

/// 重み付きの有向辺`a -(w)-> b`で`a = b w`を定義していく。
pub struct UnionFindWithPotential<T>
where
    T: Group,
{
    node: Box<[Node<T>]>,
}

impl<T> UnionFindWithPotential<T>
where
    T: Group<Set: Clone + Eq> + Copy,
{
    pub fn new(n: usize) -> Self {
        Self {
            node: vec![
                Node {
                    parent_or_size: -1,
                    potential: T::id()
                };
                n
            ]
            .into_boxed_slice(),
        }
    }

    /// `x`が所属する素集合の代表元のインデックスを返す。
    /// 新たに辺を張らない限り不変。
    ///
    /// # Time Complexity
    ///
    /// *O*(α(*N*)) amortized
    pub fn find(&mut self, x: usize) -> usize {
        let p = self.node[x].parent_or_size;
        if p.is_negative() {
            x
        } else {
            let root = self.find(p as usize);
            self.node[x] = Node {
                parent_or_size: root as i32,
                // p[self] = p[parent] * w1 = p[root] * w2 * w1
                potential: T::op(
                    self.node[p as usize].potential.clone(),
                    self.node[x].potential.clone(),
                ),
            };
            root
        }
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn size(&mut self, x: usize) -> usize {
        let r = self.find(x);
        -self.node[r].parent_or_size as usize
    }

    /// `inv(P(t)) P(s)`
    pub fn potential(&mut self, source: usize, target: usize) -> Option<T::Set> {
        if !self.same(source, target) {
            return None;
        }

        // same() 内で find() を呼ぶので、親が root になっている。
        // P(s) = P(r) w_s, P(t) = P(r) w_t より
        // P(s) = P(r) w_s = P(t) inv(w_t) w_s
        Some(T::op(
            T::inv(self.node[target].potential.clone()),
            self.node[source].potential.clone(),
        ))
    }

    /// `P(s) = P(t) * potential`の関係を追加する。
    ///
    /// - 矛盾する場合は`Err()`を返す
    /// - 矛盾していないが定義済みの場合は`Ok(false)`を返す
    /// - 新しい関係を追加した場合は`Ok(true)`を返す
    pub fn union(
        &mut self,
        source: usize,
        target: usize,
        mut potential: T::Set,
    ) -> Result<bool, ()> {
        if let Some(p) = self.potential(source, target) {
            if potential == p {
                return Ok(false);
            } else {
                return Err(());
            }
        }

        {
            let mut rs = self.find(source);
            let mut rt = self.find(target);

            // union by size
            if self.node[rs].parent_or_size > self.node[rt].parent_or_size {
                std::mem::swap(&mut rs, &mut rt);
                potential = T::inv(potential)
            }

            // P(s) = P(rs) * w_s, P(t) = P(rt) * w_t, P(s) = P(t) * potential
            // P(rs) = P(s) * inv(w_s) = P(t) * potential * inv(w_s)
            //                         = P(rt) * w_t * potential * inv(w_s)
            self.node[rs].parent_or_size += self.node[rt].parent_or_size;
            self.node[rt] = Node {
                parent_or_size: rs as i32,
                potential: T::op(
                    T::op(self.node[target].potential.clone(), potential),
                    T::inv(self.node[source].potential.clone()),
                ),
            }
        }

        Ok(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Node<T>
where
    T: Group,
{
    /// 非負なら親へのポインター、負なら要素数を表す。
    parent_or_size: i32,
    /// `inv(parent) * self`
    potential: T::Set,
}
