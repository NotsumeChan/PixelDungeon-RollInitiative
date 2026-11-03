import random


class Rng(random.Random):
    """com.watabou.utils.Random con seed propia. Mismos nombres que en Java para portar 1:1."""

    def Float(self, a: float | None = None, b: float | None = None) -> float:
        if a is None:
            return self.random()
        if b is None:
            return self.random() * a
        return a + self.random() * (b - a)

    def Int(self, a: int, b: int | None = None) -> int:
        """Int(max) -> [0, max) ; Int(min, max) -> [min, max)."""
        if b is None:
            return int(self.random() * a) if a > 0 else 0
        return a + int(self.random() * (b - a))

    def IntRange(self, a: int, b: int) -> int:
        """[a, b] inclusivo."""
        return a + int(self.random() * (b - a + 1))

    def NormalIntRange(self, a: int, b: int) -> int:
        return a + int((self.random() + self.random()) * (b - a + 1) / 2)

    def chances(self, weights: list[float]) -> int:
        value = self.Float(sum(weights))
        acc = 0.0
        for i, w in enumerate(weights):
            acc += w
            if value < acc:
                return i
        return 0

    def element(self, seq: list):
        return seq[self.Int(len(seq))] if seq else None
