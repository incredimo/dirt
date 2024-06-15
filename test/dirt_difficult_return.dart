void main() {
  print(bastard());
}

bastard() {
  if (1 < 0) {
    assert(false);
    return false;
  } else if (0 > -1) {
    assert(true);
    return true;
  } else if (!false) {
    assert(false);
    return false;
  } else {
    return false;
  }
}
