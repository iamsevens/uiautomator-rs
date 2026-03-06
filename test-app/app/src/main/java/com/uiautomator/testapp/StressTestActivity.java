package com.uiautomator.testapp;

import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;
import java.util.ArrayList;
import java.util.List;

public class StressTestActivity extends AppCompatActivity {

    private TextView tvClickCount;
    private TextView tvMemoryUsage;
    private TextView tvAnimationCount;
    private int clickCount = 0;
    private final List<Object> memoryList = new ArrayList<>();
    private final Handler handler = new Handler(Looper.getMainLooper());

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_stress_test);

        tvClickCount = findViewById(R.id.tv_click_count);
        tvMemoryUsage = findViewById(R.id.tv_memory_usage);
        tvAnimationCount = findViewById(R.id.tv_animation_count);

        setupButtons();
    }

    private void setupButtons() {
        // Rapid click test
        findViewById(R.id.btn_rapid_click).setOnClickListener(v -> {
            clickCount++;
            tvClickCount.setText("Click Count: " + clickCount);
        });

        // Memory stress test
        findViewById(R.id.btn_memory_stress).setOnClickListener(v -> {
            memoryList.clear();
            // Allocate memory
            for (int i = 0; i < 1000; i++) {
                memoryList.add(new byte[1024]); // 1KB each
            }

            // Calculate memory usage
            Runtime runtime = Runtime.getRuntime();
            long usedMemory = (runtime.totalMemory() - runtime.freeMemory()) / (1024 * 1024);
            tvMemoryUsage.setText("Memory: " + usedMemory + " MB");
        });

        // Animation stress test
        findViewById(R.id.btn_animation_stress).setOnClickListener(v -> {
            handler.removeCallbacksAndMessages(null);
            // Start multiple animations
            for (int i = 0; i < 10; i++) {
                final int index = i;
                handler.postDelayed(() -> {
                    tvAnimationCount.setText("Animations: " + (index + 1));
                }, i * 100);
            }
        });
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        handler.removeCallbacksAndMessages(null);
        memoryList.clear();
    }
}
