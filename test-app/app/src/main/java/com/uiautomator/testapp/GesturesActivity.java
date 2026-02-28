package com.uiautomator.testapp;

import android.os.Bundle;
import android.view.MotionEvent;
import android.view.View;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;

public class GesturesActivity extends AppCompatActivity {

    private TextView tvClickArea;
    private TextView tvLongClickArea;
    private TextView tvDoubleClickArea;
    private TextView tvSwipeArea;
    private View viewDrag;

    private int clickCount = 0;
    private int doubleClickCount = 0;
    private long lastClickTime = 0;
    private float startX, startY;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_gestures);

        tvClickArea = findViewById(R.id.tv_click_area);
        tvLongClickArea = findViewById(R.id.tv_long_click_area);
        tvDoubleClickArea = findViewById(R.id.tv_double_click_area);
        tvSwipeArea = findViewById(R.id.tv_swipe_area);
        viewDrag = findViewById(R.id.view_drag);

        setupGestures();
    }

    private void setupGestures() {
        // Click area
        tvClickArea.setOnClickListener(v -> {
            clickCount++;
            tvClickArea.setText("Click Area\nClick Count: " + clickCount);
        });

        // Long click area
        tvLongClickArea.setOnLongClickListener(v -> {
            tvLongClickArea.setText("Long Click Area\nLong Pressed!");
            tvLongClickArea.setBackgroundColor(0xFF4CAF50);
            tvLongClickArea.postDelayed(() -> {
                tvLongClickArea.setText("Long Click Area\nLong Press Me");
                tvLongClickArea.setBackgroundColor(0xFFFF9800);
            }, 1000);
            return true;
        });

        // Double click area
        tvDoubleClickArea.setOnClickListener(v -> {
            long currentTime = System.currentTimeMillis();
            if (currentTime - lastClickTime < 500) {
                doubleClickCount++;
                tvDoubleClickArea.setText("Double Click Area\nDouble Click Count: " + doubleClickCount);
            }
            lastClickTime = currentTime;
        });

        // Swipe area
        tvSwipeArea.setOnTouchListener((v, event) -> {
            switch (event.getAction()) {
                case MotionEvent.ACTION_DOWN:
                    startX = event.getX();
                    startY = event.getY();
                    return true;
                case MotionEvent.ACTION_UP:
                    float endX = event.getX();
                    float endY = event.getY();
                    float deltaX = endX - startX;
                    float deltaY = endY - startY;

                    String direction;
                    if (Math.abs(deltaX) > Math.abs(deltaY)) {
                        direction = deltaX > 0 ? "Right" : "Left";
                    } else {
                        direction = deltaY > 0 ? "Down" : "Up";
                    }
                    tvSwipeArea.setText("Swipe Area\nSwipe Direction: " + direction);
                    return true;
            }
            return false;
        });

        // Drag area
        viewDrag.setOnTouchListener(new View.OnTouchListener() {
            float dX, dY;

            @Override
            public boolean onTouch(View v, MotionEvent event) {
                switch (event.getAction()) {
                    case MotionEvent.ACTION_DOWN:
                        dX = v.getX() - event.getRawX();
                        dY = v.getY() - event.getRawY();
                        return true;
                    case MotionEvent.ACTION_MOVE:
                        v.animate()
                            .x(event.getRawX() + dX)
                            .y(event.getRawY() + dY)
                            .setDuration(0)
                            .start();
                        return true;
                }
                return false;
            }
        });
    }
}
