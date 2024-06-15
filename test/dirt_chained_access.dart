class A {
  A() {}

  void printHello() {
    print("Hello");
  }
}

class B {
  A a;

  B(this.a) {}
}

class C {
  B b;

  C(this.b) {}
}

void main() {
  var a = A();
  var b = B(a);
  var c = C(b);

  c.b.a.printHello();
}
