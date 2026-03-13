/** A readable signal — call to get the current value. */
export type Getter<T> = () => T;

/** Write to a signal — pass a value or an updater function. */
export type Setter<T> = (value: T | ((prev: T) => T)) => void;

/** A [getter, setter] pair returned by signal(). */
export type SignalPair<T> = [Getter<T>, Setter<T>];

/** A read-only view of a signal (getter only). */
export type ReadonlySignal<T> = Getter<T>;

/** Returned by effect() — call to stop the effect. */
export type Disposer = () => void;
