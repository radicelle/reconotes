import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.ticker import FuncFormatter
import sounddevice as sd
import threading
from collections import deque
from scipy import signal
from scipy.signal import find_peaks

class VoiceRecorderFFT:
    def __init__(self, sample_rate=44100, chunk_size=2048, buffer_duration=5):
        """
        Initialize the voice recorder with FFT visualization.
        
        Args:
            sample_rate: Sample rate in Hz
            chunk_size: Number of samples per chunk
            buffer_duration: Duration of audio buffer to display in seconds
        """
        self.sample_rate = sample_rate
        self.chunk_size = chunk_size
        self.buffer_duration = buffer_duration
        self.buffer_size = int(sample_rate * buffer_duration)
        
        # Audio buffer
        self.audio_buffer = deque(maxlen=self.buffer_size)
        self.is_recording = False
        self.stream = None
        
        # Setup plot
        self.fig, (self.ax_waveform, self.ax_fft, self.ax_peaks) = plt.subplots(3, 1, figsize=(12, 10))
        self.fig.suptitle('Voice Recorder with FFT Visualization and Peak Detection')
        
        # Waveform plot
        self.ax_waveform.set_title('Waveform')
        self.ax_waveform.set_ylabel('Amplitude')
        self.ax_waveform.set_ylim(-1, 1)
        self.line_waveform, = self.ax_waveform.plot([], [], lw=1, color='blue')
        
        # FFT plot
        self.ax_fft.set_title('FFT Magnitude Spectrum (Log Scale)')
        self.ax_fft.set_xlabel('Frequency (Hz)')
        self.ax_fft.set_ylabel('Magnitude (dB)')
        self.ax_fft.set_ylim(0, 100)
        self.ax_fft.set_xscale('log')
        self.line_fft, = self.ax_fft.plot([], [], lw=1, color='green')
        
        # Format x-axis to show Hz as regular numbers
        formatter = FuncFormatter(lambda x, p: f'{int(x):d}')
        self.ax_fft.xaxis.set_major_formatter(formatter)
        
        # Peaks plot (zoomed in on peaks)
        self.ax_peaks.set_title('Detected Peaks (Dominant Frequencies)')
        self.ax_peaks.set_xlabel('Frequency (Hz)')
        self.ax_peaks.set_ylabel('Magnitude (dB)')
        self.ax_peaks.set_xscale('log')
        self.line_peaks_spectrum, = self.ax_peaks.plot([], [], lw=1, color='green', alpha=0.5, label='Spectrum')
        self.line_peaks_markers, = self.ax_peaks.plot([], [], 'ro', markersize=8, label='Peaks')
        self.ax_peaks.legend(loc='upper right')
        
        # Format x-axis for peaks plot
        self.ax_peaks.xaxis.set_major_formatter(formatter)
        
        # Store peak annotations
        self.peak_annotations = []
        
        # Add buttons
        from matplotlib.widgets import Button
        ax_start = plt.axes([0.2, 0.02, 0.1, 0.075])
        ax_stop = plt.axes([0.35, 0.02, 0.1, 0.075])
        ax_clear = plt.axes([0.5, 0.02, 0.1, 0.075])
        
        self.btn_start = Button(ax_start, 'Start')
        self.btn_stop = Button(ax_stop, 'Stop')
        self.btn_clear = Button(ax_clear, 'Clear')
        
        self.btn_start.on_clicked(self.start_recording)
        self.btn_stop.on_clicked(self.stop_recording)
        self.btn_clear.on_clicked(self.clear_buffer)
        
        # Status text
        self.status_text = self.fig.text(0.5, -0.01, 'Ready', 
                                         ha='center', fontsize=10, color='blue')
        
    def audio_callback(self, indata, frames, time_info, status):
        """Callback for audio stream."""
        if status:
            print(f"Audio stream status: {status}")
        
        # Add audio data to buffer
        audio_data = indata[:, 0]  # Get mono channel
        self.audio_buffer.extend(audio_data)
    
    def start_recording(self, event=None):
        """Start recording."""
        if not self.is_recording:
            self.is_recording = True
            self.status_text.set_text('Recording...')
            self.status_text.set_color('red')
            
            # Start audio stream
            self.stream = sd.InputStream(
                channels=1,
                samplerate=self.sample_rate,
                blocksize=self.chunk_size,
                callback=self.audio_callback
            )
            self.stream.start()
            print("Recording started")
    
    def stop_recording(self, event=None):
        """Stop recording."""
        if self.is_recording:
            self.is_recording = False
            self.status_text.set_text('Stopped')
            self.status_text.set_color('blue')
            
            if self.stream is not None:
                self.stream.stop()
                self.stream.close()
                self.stream = None
            print("Recording stopped")
    
    def clear_buffer(self, event=None):
        """Clear the audio buffer."""
        self.audio_buffer.clear()
        self.status_text.set_text('Buffer cleared')
        print("Buffer cleared")
    
    def update_plot(self, frame):
        """Update plot with new audio data."""
        if len(self.audio_buffer) > 0:
            # Convert buffer to numpy array
            audio_data = np.array(list(self.audio_buffer))
            
            # Update waveform plot
            time_axis = np.linspace(0, len(audio_data) / self.sample_rate, len(audio_data))
            self.line_waveform.set_data(time_axis, audio_data)
            self.ax_waveform.set_xlim(0, len(audio_data) / self.sample_rate)
            
            # Compute FFT
            # Apply window to reduce spectral leakage
            windowed_data = audio_data * signal.windows.hann(len(audio_data))
            fft_result = np.fft.rfft(windowed_data)
            frequencies = np.fft.rfftfreq(len(audio_data), 1 / self.sample_rate)
            
            # Convert to dB scale
            magnitude_db = 20 * np.log10(np.abs(fft_result) + 1e-10)
            
            # Filter out near-zero frequencies (DC component) and focus on 20 Hz to Nyquist
            # This improves visibility of voice frequencies
            min_freq = 20  # Human hearing starts around 20 Hz
            freq_mask = frequencies >= min_freq
            
            frequencies_filtered = frequencies[freq_mask]
            magnitude_filtered = magnitude_db[freq_mask]
            
            # Update FFT plot
            if len(frequencies_filtered) > 0:
                self.line_fft.set_data(frequencies_filtered, magnitude_filtered)
                self.ax_fft.set_xlim(min_freq, self.sample_rate / 2)
                # Dynamic y-axis scaling with better margins
                y_min = np.percentile(magnitude_filtered, 5)
                y_max = np.max(magnitude_filtered)
                margin = (y_max - y_min) * 0.1
                self.ax_fft.set_ylim(y_min - margin, y_max + margin)
                
                # Detect peaks in the spectrum
                # Height threshold to filter out noise
                height_threshold = y_min + (y_max - y_min) * 0.2
                peaks, properties = find_peaks(magnitude_filtered, height=height_threshold, distance=10)
                
                # Update peaks plot
                if len(peaks) > 0:
                    peak_frequencies = frequencies_filtered[peaks]
                    peak_magnitudes = magnitude_filtered[peaks]
                    
                    # Sort peaks by magnitude (descending)
                    sorted_indices = np.argsort(peak_magnitudes)[::-1]
                    peak_frequencies = peak_frequencies[sorted_indices]
                    peak_magnitudes = peak_magnitudes[sorted_indices]
                    
                    # Show top 10 peaks
                    top_n = min(10, len(peak_frequencies))
                    top_peaks_freq = peak_frequencies[:top_n]
                    top_peaks_mag = peak_magnitudes[:top_n]
                    
                    # Plot spectrum and peaks
                    self.line_peaks_spectrum.set_data(frequencies_filtered, magnitude_filtered)
                    self.line_peaks_markers.set_data(top_peaks_freq, top_peaks_mag)
                    
                    self.ax_peaks.set_xlim(min_freq, self.sample_rate / 2)
                    self.ax_peaks.set_ylim(y_min - margin, y_max + margin)
                    
                    # Clear previous annotations
                    for annotation in self.peak_annotations:
                        annotation.remove()
                    self.peak_annotations.clear()
                    
                    # Add text labels for peaks with frequencies in Hz
                    for i, (freq, mag) in enumerate(zip(top_peaks_freq[:5], top_peaks_mag[:5])):
                        annotation = self.ax_peaks.annotate(
                            f'{freq:.0f}Hz',
                            xy=(freq, mag),
                            xytext=(10, 10),
                            textcoords='offset points',
                            bbox=dict(boxstyle='round,pad=0.3', facecolor='yellow', alpha=0.7),
                            arrowprops=dict(arrowstyle='->', connectionstyle='arc3,rad=0', color='red'),
                            fontsize=9,
                            fontweight='bold'
                        )
                        self.peak_annotations.append(annotation)
                    
                    # Print top peaks
                    peak_str = ', '.join([f'{f:.1f}Hz' for f in top_peaks_freq[:5]])
                    self.status_text.set_text(f'Top Peaks: {peak_str}')
                else:
                    self.line_peaks_spectrum.set_data(frequencies_filtered, magnitude_filtered)
                    self.line_peaks_markers.set_data([], [])
                    self.ax_peaks.set_xlim(min_freq, self.sample_rate / 2)
                    self.ax_peaks.set_ylim(y_min - margin, y_max + margin)
                    
                    # Clear previous annotations
                    for annotation in self.peak_annotations:
                        annotation.remove()
                    self.peak_annotations.clear()
        
        return self.line_waveform, self.line_fft, self.line_peaks_spectrum, self.line_peaks_markers
    
    def run(self):
        """Run the application."""
        # Create animation
        ani = FuncAnimation(self.fig, self.update_plot, interval=100, blit=True)
        plt.tight_layout(rect=[0, 0.1, 1, 0.98])
        plt.show()


if __name__ == '__main__':
    recorder = VoiceRecorderFFT(sample_rate=44100, chunk_size=2048, buffer_duration=5)
    recorder.run()
