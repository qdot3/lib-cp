pub struct SuffixAutomaton<T: Transition> {
    arena: Vec<Node<T>>,
}

impl<T: Transition> SuffixAutomaton<T> {
    pub fn from_str(s: &[T::Char]) -> Self {
        let mut arena = Vec::new();
        arena.push(Node::default());
        for s in s.windows(2) {}

        Self { arena }
    }

    fn append(&mut self, mut source: usize, c: T::Char, target: usize) {
        self.arena[target].max_len = self.arena[target].max_len.max(self.arena[source].max_len + 1);

        // 既存の接尾辞の末尾に文字を追加する。
        while source != 0 && self.arena[source].transition.get(c).is_none() {
            self.arena[source].transition.set(c, target);
            source = self.arena[source].link;
        }

        if let Some(source) = self.arena[source].transition.get(c) {
            debug_assert_ne!(source, target);
            // conflict!
            if self.arena[source].max_len + 1 == self.arena[target].max_len {
                // 文字 c が次に来るので、枝を張る必要がない。
                // 
                self.arena[target].link = source
            } else {
                // max_len が矛盾するので、target は使えない。新しい、source は使える。
                debug_assert!(self.arena[source].max_len < self.arena[target].max_len);
            }
        } else {
            // root から枝を生やす
            self.arena[source].transition.set(c, target);
            self.arena[target].link = source;
        }
    }
}

/// 到達可能なノードの集合が同じ部分文字列を１つのノードにまとめている。
/// 終端に至るまで文字を追加すると部分文字列は接尾辞になる。
/// 逆にこの部分文字列を接頭辞にもつすべての接尾辞がここに集約されている。
/// したがって、同値関係を定義できる。
#[derive(Debug, Clone, Default)]
struct Node<T: Transition> {
    transition: T,
    /// 同値類の最小元から頭文字を削除した部分文字列のつくる同値類へのリンク。
    /// 同値類の元は連続的なので、末尾に一文字だけ追加する場合、link をたどりつつ文字を追加すればよい。
    link: usize,
    /// 同値類の最大限の長さ
    max_len: usize,
    state: u8,
}

impl<T: Transition> Node<T> {
    /// 遷移先がない。
    const FINAL: u8 = 0b0000_0001;
    const CLONED: u8 = 0b0000_0010;
}

pub trait Transition: Clone + Default {
    type Char: Copy;

    fn get(&self, c: Self::Char) -> Option<usize>;
    fn set(&mut self, c: Self::Char, target: usize);

    fn iter(&self) -> impl IntoIterator<Item = (Self::Char, usize)>;
}
