
// id is {{ id }}

//<script src="https://ajax.googleapis.com/ajax/libs/jquery/3.7.1/jquery.min.js"></script>
// <script> 

function mermaid_animate_open_details(id, number_of_frames) {
    console.log("mermaid_animate_open_details ");
    let nbopened = 0;
    for (let i = 1; i <= number_of_frames; i++) {
        let id_full = id + "-" + i
        let div = document.getElementById(id_full);
        if (div.hidden == false) {
            nbopened = nbopened + 1;
        }
    }
    console.log("number of opened details " + nbopened);
    if (nbopened > 1) {
        for (let i = 1; i <= number_of_frames; i++) {
            let id_full = id + "-" + i
            let div = document.getElementById(id_full);
            div.hidden = true;
            div.visibility = "hidden";
            div.style.display = "none";
        }
        mermaid_animate_display_frame(id, 1, number_of_frames);
    }
};


function mermaid_animate_display_frame(id, frame, number_of_frames) {
    // console.log("display frame "+frame) ;
    let nbopened = 0;
    for (let i = 1; i <= number_of_frames; i++) {
        let id_full = id + "-" + i
        let div = document.getElementById(id_full);
        if (div.hidden == false) {
            nbopened = nbopened + 1;
        }
    }
    // console.log("number of opened details " + nbopened);
    if (nbopened > 1) {
        for (let i = 1; i <= number_of_frames; i++) {
            let id_full = id + "-" + i
            let div = document.getElementById(id_full);
            div.hidden = true;
            div.visibility = "hidden";
            div.style.display = "none";
        }
    }
    for (let i = 1; i <= number_of_frames; i++) {
        let id_full = id + "-" + i
        let div = document.getElementById(id_full);
        if (i == frame) {
            // console.log("show frame"+frame) ;
            // console.log("show id"+id) ;
            div.hidden = false;
            div.visibility = "visible";
            div.style.display = "block";
        }
        else {
            // console.log("hide") ;
            div.hidden = true;
            div.visibility = "hidden";
            div.style.display = "none";
        }
    }
};


function mermaid_animate_advance(id, number_of_frames) {
    let frame = 1;
    for (let i = 1; i <= number_of_frames; i++) {
        let id_full = id + "-" + i
        console.log("check id " + id_full);
        let div = document.getElementById(id_full);
        if (div.hidden == false) {
            // console.log("current displayed frame " + i);
            frame = i;
        }
    }

    frame = frame + 1;
    // console.log("advance to frame "+frame);
    if (frame > number_of_frames) {
        frame = 1
    }
    mermaid_animate_display_frame(id, frame, number_of_frames);
    return frame;
};

function mermaid_animate_stepback(id, number_of_frames) {
    // console.log("step back");
    let frame = 1;
    for (let i = number_of_frames; i >= 1; i--) {
        let id_full = id + "-" + i
        let div = document.getElementById(id_full);
        if (div.hidden == false) {
            // console.log("current displayed graph " + i);
            frame = i;
        }
    }

    frame = frame - 1;
    // console.log(frame);
    if (frame < 1) {
        frame = number_of_frames
    }
    mermaid_animate_display_frame(id, frame, number_of_frames);
    return frame;
};


function mermaid_animate_loop(id, number_of_frames ) {
    // console.log("animloop " + id);
    // let frame = 1;
    // var today = new Date();
    // var h = today.getHours();
    // var m = today.getMinutes();
    // var s = today.getSeconds();
    // if (s < 10) {
    //     s = "0" + s;
    // }
    // if (m < 10) {
    //     m = "0" + m;
    // }
    frame = mermaid_animate_advance(id, number_of_frames);

    // $("h1").text(h + " : " + m + " : " + s);
    is_started = $("#loop-" + id).prop('started');
    console.log("is_started=" + is_started);
    if (is_started != true) {
        console.log("stop animation loop " + id);
        return;
    }
    delay = $("#delay-" + id).val();
    console.log("delay=" + delay + " frame=" + frame);
    setTimeout(function () { mermaid_animate_loop(id, number_of_frames ) }, delay);
};


function mermaid_animate_onload() {
    console.log("mermaid_animate_onload ");
};


// (() => {
//     console.log("mermaid_animate loaded ");
//     var cusid_ele = document.getElementsByClassName('mermaid-frame');
//     for (var i = 0; i < cusid_ele.length; ++i) {
//         var item = cusid_ele[i];  
//         console.log("mermaid element " + item.id);
//         item.style.display='block' ;
//     }


// })();

window.addEventListener("load", (event) => {
    var cusid_ele = document.getElementsByClassName('mermaid-frame');
    for (var i = 1; i < cusid_ele.length; ++i) {
        var item = cusid_ele[i];  
        // console.log("mermaid element " + item.id);
        item.style.display='none' ;
    }
});

