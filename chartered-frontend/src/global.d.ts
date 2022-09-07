declare global {
    interface Window {
        extendSessionInterval: nodejs.Timer;
    }
}
