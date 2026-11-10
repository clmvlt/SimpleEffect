/**
 * First-run wizard (prompt 30): Bienvenue → Assistant IA (installed? logged in? what to do) →
 * Binaires & GPU (ffmpeg, hardware encoders, GPU, WebCodecs, face detection) → Modèles
 * (facultatif: Whisper downloads with consent) → Dossier des projets. `OnboardingDialog` is
 * mounted once by the app layout and opens by itself on the first launch of this machine;
 * « À propos » reopens it with `openOnboarding()`.
 */
export { OnboardingDialog, type OnboardingDialogProps } from './components/OnboardingDialog';
export { ONBOARDING_DONE_KEY } from './services/onboarding-steps';
export { openOnboarding, onboardingStore } from './store/onboarding-store';
