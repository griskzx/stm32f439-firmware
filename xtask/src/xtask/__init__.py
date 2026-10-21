def main() -> None:
    a = 1231
    print("Hello from xtask!")
    print("",a)
    rst = add(1,3)
    print(rst)

def add(a: int, b: int) -> int:
    rst = a + b
    return rst
