"""实验对象：微型计算器模块。

每个函数带文档字符串——提取器（extract_api.py）从中提取 API 文档。
"""


def add(a: int, b: int) -> int:
    """两数相加。

    Args:
        a: 第一个数
        b: 第二个数

    Returns:
        和
    """
    return a + b


def sub(a: int, b: int) -> int:
    """两数相减。

    Args:
        a: 被减数
        b: 减数

    Returns:
        差
    """
    return a - b


def mul(a: int, b: int) -> int:
    """两数相乘。

    Args:
        a: 第一个数
        b: 第二个数

    Returns:
        积
    """
    return a * b


def div(a: int, b: int) -> int:
    """两数相除。

    Args:
        a: 被除数
        b: 除数（非零）

    Returns:
        商
    """
    if b == 0:
        raise ValueError("除数不能为零")
    return a // b


def power(base: int, exp: int) -> int:
    """幂运算。

    Args:
        base: 底数
        exp: 指数

    Returns:
        base 的 exp 次方
    """
    return base ** exp
