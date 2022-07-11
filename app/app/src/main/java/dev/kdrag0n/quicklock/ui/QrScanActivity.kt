package dev.kdrag0n.quicklock.ui

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Size
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.camera.core.CameraSelector
import androidx.camera.core.ExperimentalGetImage
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.content.ContextCompat
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.common.InputImage
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.R
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted
import java.util.concurrent.Executors

@AndroidEntryPoint
@ExperimentalGetImage
class QrScanActivity : AppCompatActivity(R.layout.activity_qr_scan) {
    private val model: QrScanViewModel by viewModels()

    private val launcher = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        if (granted) {
            startCamera()
        } else {
            finish()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        if (checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
            startCamera()
        } else {
            launcher.launch(Manifest.permission.CAMERA)
        }

        launchStarted {
            model.pairFinishedFlow.launchCollect(this) {
                finish()
            }

            model.confirmFlow.launchCollect(this) {
                startActivity(Intent(this@QrScanActivity, ConfirmDelegationActivity::class.java))
                finish()
            }
        }
    }

    private fun startCamera() {
        val view = findViewById<PreviewView>(R.id.camera_view)
        val future = ProcessCameraProvider.getInstance(this)

        val options = BarcodeScannerOptions.Builder().run {
            setBarcodeFormats(Barcode.FORMAT_QR_CODE)
            build()
        }
        val scanner = BarcodeScanning.getClient(options)

        future.addListener({
            val cameraProvider = future.get()
            val cameraSelector = CameraSelector.Builder().run {
                requireLensFacing(CameraSelector.LENS_FACING_BACK)
                build()
            }

            val preview = Preview.Builder().run {
                build()
            }
            preview.setSurfaceProvider(view.surfaceProvider)

            val executor = Executors.newSingleThreadExecutor()
            val imageAnalysis = ImageAnalysis.Builder().run {
                setTargetResolution(Size(1280, 720))
                setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                build()
            }
            imageAnalysis.setAnalyzer(executor) { proxy ->
                proxy.image?.let { mediaImage ->
                    val img = InputImage.fromMediaImage(mediaImage, proxy.imageInfo.rotationDegrees)
                    scanner.process(img)
                        .addOnSuccessListener {
                            processBarcodes(it)
                        }
                        .addOnCompleteListener {
                            proxy.close()
                        }
                }
            }

            cameraProvider.bindToLifecycle(this, cameraSelector, preview, imageAnalysis)
        }, ContextCompat.getMainExecutor(this))
    }

    private fun processBarcodes(barcodes: List<Barcode>) {
        for (code in barcodes) {
            code.rawValue?.let {
                model.onQrScanned(it)
                return
            }
        }
    }
}
