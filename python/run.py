import crochet_py

class MyApp:
    def __init__(self):
        self.count = 0

    def run(self, cx):
        cx.label(f'Current count: {self.count}')
        if cx.button('Increment'):
            self.count += 1
        if self.count > 3:
            cx.label('You did it!')

my_app = MyApp()

crochet_py.pop_up_window(my_app.run)
