"""带有多种坏味道的示例代码"""

class OrderProcessor:
    """订单处理器 — 过大类（字段多、方法多）"""
    def __init__(self):
        self.order_id = ""
        self.user_id = ""
        self.user_name = ""
        self.user_email = ""
        self.items = []
        self.prices = []
        self.discount = 0.0
        self.tax_rate = 0.0
        self.shipping_address = ""
        self.billing_address = ""
        self.payment_method = ""
        self.payment_status = "pending"

    def process(self):
        self.validate()
        self.calculate()
        self.apply()
        self.send()

    def validate(self): pass
    def calculate(self): pass
    def apply(self): pass
    def send(self): pass
    def refund(self): pass
    def cancel(self): pass
    def archive(self): pass
    def export(self): pass
    def import_data(self): pass
    def print_report(self): pass
    def notify_user(self): pass


def compute(a, b, c, d, e, f, g):
    """过长参数列表 — 7个参数"""
    return a + b + c + d + e + f + g


def long_function_example():
    """过长函数 — 超过30行"""
    x = 1
    x = 2
    x = 3
    x = 4
    x = 5
    x = 6
    x = 7
    x = 8
    x = 9
    x = 10
    x = 11
    x = 12
    x = 13
    x = 14
    x = 15
    x = 16
    x = 17
    x = 18
    x = 19
    x = 20
    x = 21
    x = 22
    x = 23
    x = 24
    x = 25
    x = 26
    x = 27
    x = 28
    x = 29
    x = 30
    x = 31
    x = 32
    return x
