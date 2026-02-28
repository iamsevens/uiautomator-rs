package com.uiautomator.testapp;

import android.os.Bundle;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicInteger;

public class ConcurrentTestActivity extends AppCompatActivity {

    private TextView tvCounter1;
    private TextView tvCounter2;
    private TextView tvCounter3;
    private AtomicInteger counter1 = new AtomicInteger(0);
    private AtomicInteger counter2 = new AtomicInteger(0);
    private AtomicInteger counter3 = new AtomicInteger(0);
    private ExecutorService executorService = Executors.newFixedThreadPool(3);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_concurrent_test);

        tvCounter1 = findViewById(R.id.tv_counter1);
        tvCounter2 = findViewById(R.id.tv_counter2);
        tvCounter3 = findViewById(R.id.tv_counter3);

        setupButtons();
    }

    private void setupButtons() {
        // Increment all counters concurrently
        findViewById(R.id.btn_increment_all).setOnClickListener(v -> {
            executorService.execute(() -> {
                int value = counter1.incrementAndGet();
                runOnUiThread(() -> tvCounter1.setText("Counter 1: " + value));
            });

            executorService.execute(() -> {
                int value = counter2.incrementAndGet();
                runOnUiThread(() -> tvCounter2.setText("Counter 2: " + value));
            });

            executorService.execute(() -> {
                int value = counter3.incrementAndGet();
                runOnUiThread(() -> tvCounter3.setText("Counter 3: " + value));
            });
        });

        // Reset all counters
        findViewById(R.id.btn_reset_all).setOnClickListener(v -> {
            counter1.set(0);
            counter2.set(0);
            counter3.set(0);
            tvCounter1.setText("Counter 1: 0");
            tvCounter2.setText("Counter 2: 0");
            tvCounter3.setText("Counter 3: 0");
        });
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        executorService.shutdown();
    }
}
