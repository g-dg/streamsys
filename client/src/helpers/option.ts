/**
 * Attempt at implementing Rust's `Option<T>` type in Typescript.
 *
 * Some things are missing:
 *   - Support for pattern matching
 *     - `if let Some(value) = option`
 *   - Result support
 *     - `ok_or`, `ok_or_else`, `transpose` are not currently implemented
 *     - May be implemented later with exceptions or if I create a result class
 *   - Default value support
 *     - `unwrap_or_default`, `get_or_insert_default` are not included
 *     - A `default` method is provided
 *   - References
 *     - `as_ref`, `as_mut` don't exist
 *     - some functions that use references like `take_if` are less useful and behave differently
 *   - Mutable iterators
 *     - Creating a basic iterator from an Option is supported with `iter`
 *     - From iterator support may be added in the future
 *   - Fully immutable values
 *     - As with regular Javascript, keys of contained values can be modified.
 *     - To modify the root contained value, use one of `insert`, `take, `replace`, or `clone_from`
 *   - Various general Rust trait implementations
 */
export class Option<T> {
  /**
   * Used to determine if the object is an `Option`.
   * `flatten` uses this to determine if the contained value is an `Option`.
   */
  private _isOption: boolean = true;

  /**
   * `true` if a `some`, `false` if a `none`.
   */
  private _hasValue: boolean = false;

  /**
   * The value (or undefined if a `none`).
   */
  private _value: T | undefined = undefined;

  /**
   * No value.
   */
  static none<T>(): Option<T> {
    let option = new Option<T>();
    return option;
  }

  /**
   * Some value of type `T`.
   */
  static some<T>(value: T): Option<T> {
    let option = new Option<T>();
    option._value = value;
    option._hasValue = true;
    return option;
  }

  /**
   * Returns `true` if the option is a `some` value.
   */
  isSome(): boolean {
    return this._hasValue;
  }

  /**
   * Returns `true` if the option is a `some` and the value inside of it matches a predicate.
   */
  isSomeAnd(fn: (value: T) => boolean): boolean {
    if (this.isSome()) {
      return fn(this.unwrapUnchecked());
    } else {
      return false;
    }
  }

  /**
   * Returns `true` if the option is a `none` value.
   */
  isNone(): boolean {
    return !this._hasValue;
  }

  /**
   * Returns the contained `some` value.
   * Throws the error provided by `msg`
   */
  expect(err: any): T {
    if (this.isSome()) {
      return this.unwrapUnchecked();
    } else {
      throw err;
    }
  }

  /**
   * Returns the contained `some` value.
   * Throws a `TypeError` if the value is `none`.
   */
  unwrap(): T {
    if (this.isSome()) {
      return this.unwrapUnchecked();
    } else {
      throw new TypeError();
    }
  }

  /**
   * Returns the contained `some` value or a provided default.
   */
  unwrapOr(defaultValue: T): T {
    if (this.isSome()) {
      return this.unwrapUnchecked();
    } else {
      return defaultValue;
    }
  }

  /**
   * Returns the contained `some` value or computes it from the provided function.
   */
  unwrapOrElse(fn: () => T): T {
    if (this.isSome()) {
      return this.unwrapUnchecked();
    } else {
      return fn();
    }
  }

  /**
   * Returns the contained `some` value, without checking that the value is not `none`.
   * Calling this method on a `none` value results in undefined behaviour.
   */
  unwrapUnchecked(): T {
    return this._value as T;
  }

  /**
   * Maps an `Option<T>` to `Option<U>` by applying a function to a contained value (if `some`) or returns `none` (if `none`).
   */
  map<U>(fn: (value: T) => U): Option<U> {
    if (this.isSome()) {
      return Option.some(fn(this.unwrapUnchecked()));
    } else {
      return Option.none();
    }
  }

  /**
   * Calls a function with the contained value if `some`.
   * Returns the value.
   */
  inspect(fn: (value: T) => void): Option<T> {
    if (this.isSome()) {
      fn(this.unwrapUnchecked());
    }
    return this;
  }

  /**
   * Returns the provided default result (if `none`), or applies a function to the contained value (if `some`).
   */
  mapOr<U>(defaultValue: U, fn: (value: T) => U): U {
    if (this.isSome()) {
      return fn(this.unwrapUnchecked());
    } else {
      return defaultValue;
    }
  }

  /**
   * Computes a default function result (if `none`), or applies a different function to the contained value (if `some`).
   */
  mapOrElse<U>(defaultFn: () => U, fn: (value: T) => U): U {
    if (this.isSome()) {
      return fn(this.unwrapUnchecked());
    } else {
      return defaultFn();
    }
  }

  /**
   * Returns an iterator over the possibly contained value.
   */
  iter(): Iterable<T> {
    return new OptionIterator(this);
  }

  /**
   * Returns `none` if the option is `none, otherwise returns `optb`.
   */
  and<U>(optb: Option<U>): Option<U> {
    if (this.isSome()) {
      return optb;
    } else {
      return Option.none();
    }
  }

  /**
   * Returns `none` if the option is `none`, otherwise calls `fn` with the wrapped value and returns the result.
   * Some languages call this operation flatmap.
   */
  andThen<U>(fn: (value: T) => Option<U>): Option<U> {
    if (this.isSome()) {
      return fn(this.unwrapUnchecked());
    } else {
      return Option.none();
    }
  }

  /**
   * Returns `none` if the option is `none`, otherwise calls `predicate` with the wrapped value and returns:
   *   - `some<T>` if `predicate` returns `true` (where `T` is the wrapped value),
   *   - `none` if predicate returns false.
   */
  filter(predicate: (value: T) => boolean): Option<T> {
    if (this.isSome()) {
      if (predicate(this.unwrapUnchecked())) {
        return this;
      } else {
        return Option.none();
      }
    } else {
      return Option.none();
    }
  }

  /**
   * Returns the option if it contains a value, otherwise returns `optb`.
   */
  or(optb: Option<T>): Option<T> {
    if (this.isSome()) {
      return this;
    } else {
      return optb;
    }
  }

  /**
   * Returns the option if it contains a value, otherwise calls `fn` and returns the result.
   */
  orElse(fn: () => Option<T>): Option<T> {
    if (this.isSome()) {
      return this;
    } else {
      return fn();
    }
  }

  /**
   * Returns `some` if exactly one of `this`, `optb` is `some`, otherwise returns `none`.
   */
  xor(optb: Option<T>): Option<T> {
    if (this.isSome() == optb.isSome()) {
      return Option.none();
    } else {
      if (this.isSome()) {
        return this;
      } else {
        return optb;
      }
    }
  }

  /**
   * Inserts `value` into the option, then returns it.
   */
  insert(value: T): T {
    this._value = value;
    this._hasValue = true;
    return this.unwrapUnchecked();
  }

  /**
   * Inserts `value` into the option if it is `None`, then returns the contained value.
   */
  getOrInsert(value: T): T {
    if (this.isSome()) {
      return this.unwrapUnchecked();
    } else {
      this._value = value;
      this._hasValue = true;
      return this.unwrapUnchecked();
    }
  }

  /**
   * Inserts a value computed from `fn` into the option if it is `none`, then returns the contained value.
   */
  getOrInsertWith(fn: () => T): T {
    if (this.isSome()) {
      return this.unwrapUnchecked();
    } else {
      this._value = fn();
      this._hasValue = true;
      return this.unwrapUnchecked();
    }
  }

  /**
   * Takes the value out of the option, leaving a `none` in its place, then returns the original value.
   */
  take(): Option<T> {
    const oldHasValue = this.isSome();
    const oldValue = oldHasValue ? this.unwrapUnchecked() : undefined;

    this._hasValue = false;
    this._value = undefined;

    if (oldHasValue) {
      return Option.some(oldValue!);
    } else {
      return Option.none();
    }
  }

  /**
   * Takes the value out of the option and returns it, but only if the predicate evaluates to true when passed the value.
   * Otherwise, returns `none`
   */
  takeIf(predicate: (value: T) => boolean): Option<T> {
    if (this.isSome()) {
      if (predicate(this.unwrapUnchecked())) {
        return this.take();
      } else {
        return Option.none();
      }
    } else {
      return Option.none();
    }
  }

  /**
   * Replaces the value in the option with the value in the parameter, returning the previous value.
   */
  replace(value: T): Option<T> {
    const oldHasValue = this.isSome();
    const oldValue = oldHasValue ? this.unwrapUnchecked() : undefined;

    this._value = value;
    this._hasValue = true;

    if (oldHasValue) {
      return Option.some(oldValue!);
    } else {
      return Option.none();
    }
  }

  /**
   * Zips `this` with another `Option`.
   * If `this` is `some(t)` and `other` is `some(o)`, this method returns `some([t, o])`. Otherwise, `none is returned.
   */
  zip<U>(other: Option<U>): Option<[T, U]> {
    if (this.isSome() && other.isSome()) {
      return Option.some([this.unwrapUnchecked(), other.unwrapUnchecked()]);
    } else {
      return Option.none();
    }
  }

  /**
   * Zips `this` and another `Option` with function `fn`.
   * The function is called with the contained values of both `Option`s and its return value is returned.
   * If any of the values are `none`, then `none` is returned.
   */
  zipWith<U, R>(
    other: Option<U>,
    fn: (thisValue: T, otherValue: U) => R
  ): Option<R> {
    if (this.isSome() && other.isSome()) {
      return Option.some(fn(this.unwrapUnchecked(), other.unwrapUnchecked()));
    } else {
      return Option.none();
    }
  }

  /**
   * Unzips an option containing an array of two options.
   * If `this` is `some([a, b])`, this method returns `[some(a), some(b)]. Otherwise, `[None, None]` is returned.
   *
   * Note: This function does not have proper compile-time type checking and does type checking at runtime.
   * If the contained value is not an array with 2 items, it throws a `TypeError`.
   */
  unzip<U, V>(): [Option<U>, Option<V>] {
    if (this.isSome()) {
      const value = this.unwrapUnchecked();
      if (
        typeof value == "object" &&
        Array.isArray(value) &&
        value.length == 2
      ) {
        return [Option.some(value[0]), Option.some(value[1])];
      } else {
        throw new TypeError();
      }
    }
    return [Option.none(), Option.none()];
  }

  /**
   * Returns a `structuredClone` of the object
   */
  copied(): Option<T> {
    return structuredClone(this);
  }

  /**
   * Returns a `structuredClone` of the object
   */
  cloned(): Option<T> {
    return structuredClone(this);
  }

  /**
   * Converts from an `Option<Option<T>>` to an `Option<T>`.
   *
   * Note: This function does not have proper compile-type type checking and does type checking at runtime.
   * If the contained value is not an `Option`, it throws a `TypeError`.
   */
  flatten(): Option<T> {
    if (this.isSome()) {
      const value = this.unwrapUnchecked();
      if ((value as any)?._isOption) {
        return value as Option<T>;
      } else {
        throw new TypeError();
      }
    } else {
      return Option.none();
    }
  }

  /**
   * Returns a shallow clone of the `Option`.
   */
  clone(): Option<T> {
    if (this.isSome()) {
      return Option.some(this.unwrapUnchecked());
    } else {
      return Option.none();
    }
  }

  /**
   * Sets this `Option` to contain a shallow clone of another `Option`.
   */
  clone_from(source: Option<T>): void {
    this._hasValue = source.isSome();
    this._value = source.unwrapUnchecked();
  }

  /**
   * Returns the default value of `Option.none`.
   */
  default(): Option<T> {
    return Option.none();
  }
}

/**
 * An iterator over an `Option<T>`.
 *
 * If the `Option` is `some`, then the value is returned only on the first call of `next`.
 * All other calls of `next` return the end of the iterator.
 */
export class OptionIterator<T> implements Iterator<T, Option<T>> {
  constructor(option: Option<T>) {
    this._option = option;
  }

  private _option: Option<T>;
  private _returned = false;

  next(): IteratorResult<T, Option<T>> {
    if (!this._returned) {
      this._returned = true;
      if (this._option.isSome()) {
        return { value: this._option.unwrapUnchecked(), done: false };
      }
    }
    return { value: this._option, done: true };
  }

  [Symbol.iterator]() {
    return this;
  }
}
