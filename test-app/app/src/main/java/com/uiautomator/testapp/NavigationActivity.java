package com.uiautomator.testapp;

import android.content.Intent;
import android.os.Bundle;
import android.widget.Button;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;

public class NavigationActivity extends AppCompatActivity {

    private int pageNumber = 1;
    private TextView tvPageInfo;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_navigation);

        pageNumber = getIntent().getIntExtra("page_number", 1);
        tvPageInfo = findViewById(R.id.tv_page_info);
        tvPageInfo.setText("Page: " + pageNumber);

        setupButtons();
    }

    private void setupButtons() {
        // Next page button
        findViewById(R.id.btn_next_page).setOnClickListener(v -> {
            Intent intent = new Intent(this, NavigationActivity.class);
            intent.putExtra("page_number", pageNumber + 1);
            startActivity(intent);
        });

        // Back button
        findViewById(R.id.btn_back).setOnClickListener(v -> finish());
    }
}
