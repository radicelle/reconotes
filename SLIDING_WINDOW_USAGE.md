# Guide Utilisation - Fenêtre Glissante

## Démarrage

### 1. Lancer le Backend

```powershell
cd 'c:\Users\manua\CodeProjects\other\diapazon\recognotes-rust-backend'
cargo run --release
```

Le backend doit être accessible sur `http://localhost:5000`

### 2. Lancer l'Application GUI

```powershell
cd 'c:\Users\manua\CodeProjects\other\diapazon\recognotes-desktop-gui'
cargo run --release
```

### 3. Test Audio

Cliquez sur **"Start Recording"** et commencez à jouer des notes.

## Observations Attendues

### ✅ Bon Fonctionnement

1. **Initialisation**: L'app attend ~1 seconde avant de commencer l'analyse
   - Normal! Le buffer glissant accumule 48,000 samples

2. **Analyse Continue**: Une fois prête, l'analyse s'exécute toutes les 20ms
   - Vous devriez voir des notes s'afficher en temps quasi-réel
   - L'écran se met à jour fluide

3. **Logs**:
   ```
   Waiting for audio buffer: 5000/48000 samples
   Waiting for audio buffer: 10000/48000 samples
   ...
   🎵 Received 3 notes from backend
      - C4 (95% confidence)
      - E4 (92% confidence)
      - G4 (88% confidence)
   ```

4. **Meilleure Résolution**: 
   - Fréquences basses mieux détectées
   - Plus stable et cohérent
   - Moins de "jitter" entre les notes

## Dépannage

### 🔴 Pas de notes détectées après 1 seconde

1. **Vérifier le backend:**
   ```powershell
   curl http://localhost:5000/health
   ```
   Devrait retourner `200 OK`

2. **Vérifier les logs du GUI:**
   - Les logs devraient montrer "Received X notes"
   - Si erreur réseau, vérifier l'adresse du backend

3. **Vérifier le audio input:**
   - Augmentez le volume
   - Essayez un autre appareil audio

### ⏱️ Latence trop élevée

- L'intervalle est actuellement **20ms** entre analyses
- Si c'est trop lent, on peut réduire à **10ms** (plus d'analyses)
- Si c'est trop rapide, augmenter à **30ms** ou **50ms**

**Modifier dans `main.rs`:**
```rust
sliding_window_interval: std::time::Duration::from_millis(20),  // ← Changer ici
```

## Comparaison Avant/Après

### Avant (10ms chunks)
```
[====] Audio: 10ms
[==============] Backend: send
[========================] Analyse: 100 fois/sec
[Résultat] Fréquence basse ~200Hz
```

### Après (20ms sliding window)
```
[================================================] Audio: 1 seconde
[========================] Analyse: 50 fois/sec
[==============] Backend: send 48KB complet
[Résultat] Fréquence basse ~20Hz, beaucoup plus stable
```

## Métriques de Performance

À surveiller dans les logs:

```
Backend response in 18ms: 5 notes from 96KB audio
```

Cela signifie:
- **18ms**: Temps de traitement au backend
- **5 notes**: Détectées
- **96KB**: Taille du buffer (1 seconde à 48kHz)

**Idéal**: < 25ms pour rester au-dessus de 40 FPS de display

## Configuration Recommandée

| Paramètre | Valeur | Raison |
|-----------|--------|--------|
| Buffer Size | 1 second | Bonne résolution FFT |
| Interval | 20ms | Balance latence/CPU |
| Sample Rate | 48kHz | Windows standard |

---

**Vous êtes prêt! 🎉**
