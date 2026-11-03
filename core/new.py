class New():
    def booleanList(LENGTH) -> list[bool]:
        return [False for _ in range(LENGTH)]

    def integerList(LENGTH) -> list[int]:
        return [0 for _ in range(LENGTH)]
    
    def fill(iterable, data) -> list:
        return [data for _ in range(len(iterable))]